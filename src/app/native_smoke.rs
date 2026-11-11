//! Opt-in replay through the real native window, renderer and input handlers.
//! This is distinct from OS input injection and cannot verify a system IME UI.
use super::*;
use markion::selection::{CellSelectionAxis, SelectionState};
use std::sync::atomic::{AtomicBool, Ordering};

type SmokeResult<T = ()> = Result<T, String>;

fn ensure(condition: bool, message: &str) -> SmokeResult {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

async fn frame(
    this: &gpui::WeakEntity<MarkionApp>,
    cx: &mut gpui::AsyncWindowContext,
) -> SmokeResult {
    let painted = Arc::new(AtomicBool::new(false));
    let flag = painted.clone();
    this.update_in(cx, |_, window, cx| {
        window.on_next_frame(move |_, _| flag.store(true, Ordering::Release));
        cx.notify();
    })
    .map_err(|error| error.to_string())?;
    for _ in 0..400 {
        cx.background_executor()
            .timer(Duration::from_millis(5))
            .await;
        if painted.load(Ordering::Acquire) {
            return Ok(());
        }
    }
    Err("native window did not complete a requested frame within two seconds".into())
}

async fn settle(
    this: &gpui::WeakEntity<MarkionApp>,
    cx: &mut gpui::AsyncWindowContext,
) -> SmokeResult {
    frame(this, cx).await?;
    frame(this, cx).await
}

impl MarkionApp {
    pub(super) fn start_native_visual_smoke(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(directory) = env::var_os("MARKION_NATIVE_SMOKE_DIR").map(PathBuf::from) else {
            return;
        };
        // The launcher supplies an isolated profile and repository fixtures.
        // Never save or transmit the replay's in-memory document changes.
        self.auto_save_preferences.enabled = false;
        self.check_for_updates_on_startup = false;
        self.sidebar_visible = false;
        self.view_mode = ViewMode::VisualEdit;
        window.focus(&self.focus_handle);
        window.activate_window();
        let original_clipboard = cx.read_from_clipboard();
        cx.spawn_in(window, async move |this, cx| {
            let mut observations = Vec::new();
            let result = replay(&this, cx, &directory, &mut observations).await;
            let report = serde_json::json!({
                "passed": result.is_ok(), "error": result.err(),
                "platform": process_footprint_platform(), "version": env!("CARGO_PKG_VERSION"),
                "evidence": "real native GPUI window and paint; application input callbacks; no OS mouse/keyboard/IME injection",
                "observations": observations,
            });
            let output = directory.join("native-smoke.json");
            if let Ok(json) = serde_json::to_vec_pretty(&report) {
                if let Err(error) = fs::write(&output, json) {
                    tracing::error!(target: "markion::performance", %error, "native smoke report write failed");
                }
            }
            tracing::info!(target: "markion::performance", passed = report["passed"].as_bool(), output = ?output, "native visual smoke finished");
            let _ = this.update_in(cx, |_, _, cx| {
                cx.write_to_clipboard(
                    original_clipboard.unwrap_or_else(|| ClipboardItem::new_string(String::new())),
                );
                cx.quit();
            });
        }).detach();
    }
}

async fn replay(
    this: &gpui::WeakEntity<MarkionApp>,
    cx: &mut gpui::AsyncWindowContext,
    directory: &Path,
    observations: &mut Vec<serde_json::Value>,
) -> SmokeResult {
    ensure(
        super::performance::enabled(),
        "MARKION_PROFILE_VISUAL_INPUT must also be enabled",
    )?;
    settle(this, cx).await?;
    for kb in [100, 300, 600, 1024] {
        super::performance::take_samples();
        let path = directory.join(format!("mixed-{kb}kb.md"));
        this.update_in(cx, |app, window, cx| {
            app.open_supported_path(path, OpenPathIntent::ReplaceActive, cx)
                .map_err(|error| error.to_string())?;
            window.focus(&app.focus_handle);
            Ok::<_, String>(())
        })
        .map_err(|error| error.to_string())??;
        settle(this, cx).await?;
        this.update_in(cx, |app, _, cx| {
            let blocks = app.active_tab().document.visual_blocks_shared();
            let middle = app.active_tab().document.text().len() / 2;
            let (index, block) = blocks
                .iter()
                .enumerate()
                .find(|(_, block)| {
                    block.source_range.start >= middle
                        && matches!(block.kind, VisualBlockKind::Paragraph)
                })
                .ok_or("fixture has no middle paragraph")?;
            let offset = block
                .editable_runs
                .first()
                .ok_or("paragraph has no payload")?
                .content_range
                .start;
            app.move_to(offset, cx);
            app.active_tab().visual_list.scroll_to_reveal_item(index);
            Ok::<_, String>(())
        })
        .map_err(|error| error.to_string())??;
        settle(this, cx).await?;
        for _ in 0..20 {
            this.update_in(cx, |app, window, cx| {
                EntityInputHandler::replace_text_in_range(app, None, "a", window, cx);
            })
            .map_err(|error| error.to_string())?;
            settle(this, cx).await?;
            this.update_in(cx, |app, _, _| {
                ensure(
                    app.active_tab().visual_input_profile.is_none(),
                    "input did not produce a current visible caret paint",
                )
            })
            .map_err(|error| error.to_string())??;
        }
        let memory = ProcessFootprint::sample();
        observations.push(serde_json::json!({ "fixture_kb": kb,
            "resident_bytes": memory.resident_current, "commit_bytes": memory.commit_current,
            "paint_samples": super::performance::take_samples() }));
    }

    let path = directory.join("native-p3-smoke.md");
    let original = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    this.update_in(cx, |app, window, cx| {
        app.open_supported_path(path, OpenPathIntent::ReplaceActive, cx)?;
        window.focus(&app.focus_handle);
        let blocks = app.active_tab().document.visual_blocks_shared();
        let paragraph = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .ok_or("missing paragraph")?;
        app.select_source_blocks(paragraph, false, cx);
        app.navigate_source_selection(true, true, false, cx);
        let Some(SelectionState::Blocks(selection)) =
            app.active_tab().structured_selection.as_ref()
        else {
            return Err("native block selection is missing".into());
        };
        let expected = selection
            .clipboard(&app.active_tab().document)
            .map_err(|error| format!("native copy payload: {error:?}"))?;
        app.copy_source_selection(cx);
        let clipboard = cx
            .read_from_clipboard()
            .ok_or("native clipboard is empty")?;
        ensure(
            clipboard.text().as_deref() == Some(expected.plain.as_str()),
            "native clipboard did not retain exact Unicode text",
        )?;
        ensure(
            clipboard.html() == Some(expected.html.as_str()),
            "native clipboard did not retain exact HTML",
        )?;
        Ok::<_, String>(())
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    this.update_in(cx, |app, _, cx| {
        app.delete_source_selection(cx);
    })
    .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| app.undo(&Undo, window, cx))
        .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, _, _| {
        ensure(
            app.active_tab().document.text() == original,
            "native block undo lost source",
        )?;
        ensure(
            matches!(
                app.active_tab().structured_selection,
                Some(SelectionState::Blocks(_))
            ),
            "native undo lost block selection",
        )
    })
    .map_err(|error| error.to_string())??;
    observations.push(serde_json::json!({"case":"block selection, native text/HTML clipboard, delete and undo", "passed":true}));

    this.update_in(cx, |app, _, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .ok_or("missing table")?;
        app.select_table_cells(index, (1, 0), CellSelectionAxis::Rectangle, false, cx);
        app.select_table_cells(index, (2, 1), CellSelectionAxis::Rectangle, true, cx);
        ensure(
            app.paste_table_matrix("矩阵一\t😀\n矩阵二\tvalue", cx),
            "matrix paste was not handled",
        )
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| app.undo(&Undo, window, cx))
        .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, _, cx| {
        ensure(
            app.active_tab().document.text() == original,
            "matrix undo lost source",
        )?;
        let blocks = app.active_tab().document.visual_blocks_shared();
        let (index, field) = blocks
            .iter()
            .enumerate()
            .find_map(|(index, block)| {
                let VisualBlockEditor::Table { cells } = block.editor.as_ref()? else {
                    return None;
                };
                Some((index, cells.last()?.field.clone()))
            })
            .ok_or("missing final table cell")?;
        app.move_to(field.source_range.end, cx);
        app.active_tab().visual_list.scroll_to_reveal_item(index);
        Ok::<_, String>(())
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| app.indent(&Indent, window, cx))
        .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| {
        let field = app
            .active_tab()
            .document
            .visual_editor_field_at(&app.active_tab().selected_range)
            .ok_or("Tab lost its cell")?;
        ensure(
            matches!(
                field.kind,
                VisualEditorFieldKind::TableCell { column: 0, .. }
            ),
            "final Tab did not enter the new row",
        )?;
        app.undo(&Undo, window, cx);
        Ok::<_, String>(())
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    observations.push(serde_json::json!({"case":"native table rectangle, matrix paste, final-cell Tab and undo", "passed":true}));

    let (code_index, code_target, payload_offset) = this
        .update_in(cx, |app, _, cx| {
            let blocks = app.active_tab().document.visual_blocks_shared();
            let (index, block, payload) = blocks
                .iter()
                .enumerate()
                .find_map(|(index, block)| {
                    let VisualBlockEditor::Code { payload, .. } = block.editor.as_ref()? else {
                        return None;
                    };
                    (block.list_depth == 0).then_some((index, block, payload))
                })
                .ok_or("missing ordinary code")?;
            let target = BlockTarget::from_block(app.active_tab().document.version(), block);
            let offset = payload.source_range.start;
            app.move_to(offset, cx);
            app.active_tab().visual_list.scroll_to_reveal_item(index);
            Ok::<_, String>((index, target, offset))
        })
        .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    let before = this
        .update_in(cx, |app, _, _| {
            app.active_tab()
                .visual_list
                .bounds_for_item(code_index)
                .map(|bounds| f32::from(bounds.size.height))
        })
        .map_err(|error| error.to_string())?
        .ok_or("code was not laid out in the native window")?;
    this.update_in(cx, |app, _, cx| {
        app.toggle_visual_code_fold(code_target, cx)
    })
    .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, _, cx| {
        let height = app
            .active_tab()
            .visual_list
            .bounds_for_item(code_index)
            .map(|bounds| f32::from(bounds.size.height))
            .ok_or("folded code has no native bounds")?;
        ensure(
            height < before,
            "folding did not reduce the native row height",
        )?;
        app.move_to(payload_offset, cx);
        ensure(
            app.active_tab().folded_visual_code_blocks.is_empty(),
            "navigation did not expand code",
        )
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    observations.push(serde_json::json!({"case":"native code folding geometry and navigation expansion", "passed":true}));

    this.update_in(cx, |app, _, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .ok_or("missing IME paragraph")?;
        let offset = blocks[index]
            .editable_runs
            .first()
            .ok_or("missing IME payload")?
            .content_range
            .start;
        app.move_to(offset, cx);
        app.active_tab().visual_list.scroll_to_reveal_item(index);
        Ok::<_, String>(())
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| {
        EntityInputHandler::replace_and_mark_text_in_range(
            app,
            None,
            "中文",
            Some(2..2),
            window,
            cx,
        )
    })
    .map_err(|error| error.to_string())?;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| {
        ensure(
            app.active_tab().marked_range.is_some(),
            "native IME callback lost marked range",
        )?;
        ensure(
            app.active_tab().visual_marked_range_bounds.is_some(),
            "native IME callback has no candidate geometry",
        )?;
        EntityInputHandler::replace_text_in_range(app, None, "中文", window, cx);
        Ok::<_, String>(())
    })
    .map_err(|error| error.to_string())??;
    settle(this, cx).await?;
    this.update_in(cx, |app, window, cx| {
        app.undo(&Undo, window, cx);
        ensure(
            app.active_tab().document.text() == original,
            "native composition undo lost source",
        )
    })
    .map_err(|error| error.to_string())??;
    observations.push(serde_json::json!({"case":"IME callbacks, real shaped candidate bounds and composition undo", "passed":true, "system_candidate_window_tested":false}));
    settle(this, cx).await?;
    Ok(())
}
