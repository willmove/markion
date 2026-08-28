//! Deterministic mutation-integrity coverage for the checked canonical
//! boundary: multi-tab isolation (task 4.2), the §1.1–§1.9 state machine
//! (tasks 4.3/4.4 and 5.1), render-only invariants (task 4.5), and the named
//! regression for the duplicated-headings incident class (task 5.2).

use super::application::ExternalCheckRequest;
use super::state::StartupOpenIntent;
use super::*;
use gpui::{EntityInputHandler, TestAppContext};

fn incident_fixture() -> String {
    let mut text = String::from("# Notes\n");
    for n in 1..=9 {
        text.push_str(&format!("\n## §1.{n}\n\nbody {n}\n"));
    }
    text.push_str("\n| a | b |\n|---|---|\n| 1 | 2 |\n");
    text
}

fn heading_line_count(text: &str) -> usize {
    text.lines()
        .filter(|line| line.trim_start().starts_with('#'))
        .count()
}

fn focus_and_park(app: &gpui::Entity<MarkionApp>, cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
}

fn type_text(app: &gpui::Entity<MarkionApp>, cx: &mut gpui::VisualTestContext, text: &str) {
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_text_in_range(app, None, text, window, cx)
        });
    });
    cx.run_until_parked();
}

fn type_text_in_range(
    app: &gpui::Entity<MarkionApp>,
    cx: &mut gpui::VisualTestContext,
    utf16_range: std::ops::Range<usize>,
    text: &str,
) {
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_text_in_range(app, Some(utf16_range), text, window, cx)
        });
    });
    cx.run_until_parked();
}

fn ime_mark(app: &gpui::Entity<MarkionApp>, cx: &mut gpui::VisualTestContext, text: &str) {
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(app, None, text, None, window, cx)
        });
    });
    cx.run_until_parked();
}

fn ime_cancel(app: &gpui::Entity<MarkionApp>, cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::unmark_text(app, window, cx)
        });
    });
    cx.run_until_parked();
}

/// Simulate the platform querying the selection: this is what refreshes the
/// generation the platform's future explicit offsets are validated against.
fn query_selection(app: &gpui::Entity<MarkionApp>, cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        let _ = app.update(cx, |app, cx| {
            EntityInputHandler::selected_text_range(app, false, window, cx)
        });
    });
}

/// Task 5.2 — named regression for the duplicated-headings incident class.
///
/// Violated precondition (pre-fix): `restore_snapshot` assigned the snapshot
/// document wholesale, resurrecting the snapshot's older `text_version`. The
/// tab-level per-version caches (`display_text_cache`, source layout,
/// measured height) assume version numbers are globally unique per text, so
/// after undo + one more edit the same version number named two different
/// texts and the editor served the first epoch's cached text while the
/// canonical layer held another. Platform input offsets derived from that
/// aliased generation were then reinterpreted (and, pre-boundary, clamped)
/// into the wrong place — the unauthorized-duplication class that surfaced
/// as every heading §1.1–§1.9 appearing twice in canonical memory while the
/// file on disk stayed clean.
///
/// Minimal pre-fix operation trace (red on the pre-fix code):
///   1. render/share text            (v1 → display cache holds T1)
///   2. type "X" and share text      (v2 → display cache holds T2)
///   3. undo                          [pre-fix: version REVERTS to v1]
///   4. type "Y"                      [pre-fix: version v2 again, text T3 ≠ T2]
///   5. share text                    [pre-fix: stale "T2" served for T3]
#[gpui::test]
fn undo_restore_cannot_reuse_versions_or_alias_display_text(cx: &mut TestAppContext) {
    let fixture = incident_fixture();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(fixture.clone()))];
        app.view_mode = ViewMode::Edit;
        app
    });
    focus_and_park(&app, cx);

    // Every version this tab's document has ever exposed, mapped to the text
    // it named. A version may never name two different texts.
    let mut seen: std::collections::HashMap<u64, String> = std::collections::HashMap::new();
    let observe = |app: &gpui::Entity<MarkionApp>,
                   cx: &mut gpui::VisualTestContext,
                   seen: &mut std::collections::HashMap<u64, String>| {
        app.update(cx, |app, _| {
            let tab = app.active_tab();
            let version = tab.document.version();
            let text = tab.document.text().to_string();
            if let Some(previous) = seen.insert(version, text.clone()) {
                assert_eq!(
                    previous, text,
                    "version {version} named two different texts in one document"
                );
            }
            let shared = tab.shared_document_text();
            assert_eq!(
                shared.as_ref(),
                text,
                "per-version display cache must serve the canonical text"
            );
        });
    };

    // Step 1: populate the display cache at v1.
    observe(&app, cx, &mut seen);

    // Step 2: type, then cache v2's display text.
    app.update(cx, |app, _| {
        let caret = app.active_tab().document.text().len();
        app.active_tab_mut().selected_range = caret..caret;
    });
    type_text(&app, cx, "X");
    observe(&app, cx, &mut seen);
    app.update(cx, |app, _| {
        assert!(app.active_tab().document.text().ends_with('X'));
    });

    // Step 3: undo. The restored text must carry a NEW version.
    let version_before_undo = app.update(cx, |app, _| app.active_tab().document.version());
    cx.dispatch_action(Undo);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let version = app.active_tab().document.version();
        assert!(
            version > version_before_undo,
            "undo must advance the version, not revert it to a number that \
             already named different text (pre-fix root cause)"
        );
    });
    observe(&app, cx, &mut seen);

    // Step 4: type again — the recomposed version must not collide with the
    // v2 that already named "…X…".
    app.update(cx, |app, _| {
        let caret = app.active_tab().document.text().len();
        app.active_tab_mut().selected_range = caret..caret;
    });
    type_text(&app, cx, "Y");
    observe(&app, cx, &mut seen);

    // The incident signature itself: no heading line may have been
    // duplicated by any step of the sequence.
    app.update(cx, |app, _| {
        assert_eq!(
            heading_line_count(app.active_tab().document.text()),
            heading_line_count(&fixture)
        );
    });
}

/// Task 4.5 — caret/selection/scroll/derived reads never create mutation
/// entries, never change the version, and keep the per-version caches intact.
#[gpui::test]
fn render_only_interaction_creates_no_mutations(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            incident_fixture(),
        ))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    focus_and_park(&app, cx);

    let (version, journal_len) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (tab.document.version(), tab.document.mutation_journal_len())
    });

    app.update(cx, |app, cx| {
        app.move_to(14, cx);
        app.select_to(30, cx);
        let tab = app.active_tab_mut();
        tab.editor_scroll.set_offset(point(px(0.), px(-40.)));
        tab.visual_caret_bounds = None;
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        // Derived reads: per-version caches populate but nothing mutates.
        let visual = tab.document.visual_blocks_shared();
        let preview = tab.document.preview_blocks_shared();
        let outline = tab.document.outline();
        let stats = tab.document.stats();
        let text = tab.shared_document_text();
        assert_eq!(outline.len(), 10);
        assert!(stats.words > 0);
        assert!(!text.is_empty());
        assert!(Arc::ptr_eq(&visual, &tab.document.visual_blocks_shared()));
        assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.mutation_journal_len(), journal_len);
        assert_eq!(tab.document.text(), incident_fixture());
    });
}

/// Task 4.2 — a reload read completing after intervening edits (here: an
/// undo that left the document clean again) cannot overwrite the newer
/// canonical state, and the rejection is attributable without content.
#[gpui::test]
fn delayed_reload_cannot_overwrite_intervening_edits(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("late.md");
    fs::write(&path, "disk one\n").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.view_mode = ViewMode::Edit;
        app
    });
    focus_and_park(&app, cx);

    // Capture the round exactly as check_external_changes would.
    let request = app.update(cx, |app, _| {
        let tab = app.active_tab().document_tab().unwrap();
        ExternalCheckRequest {
            recovery_id: tab.recovery_id,
            path: path.clone(),
            known: tab.document.disk_identity().cloned(),
            read_for_reload: true,
            instance: tab.document.instance_id(),
            version: tab.document.version(),
        }
    });

    // Intervening edit plus undo: the text rounds back to the on-disk bytes
    // (so the document is clean and the identity still matches), but the
    // version has moved on — only the generation guard can reject the
    // delayed reload now.
    type_text(&app, cx, "typed\n");
    cx.dispatch_action(Undo);
    cx.run_until_parked();

    let (before_text, before_version) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(!tab.document.is_dirty(), "undo restored the clean text");
        (tab.document.text().to_string(), tab.document.version())
    });

    app.update(cx, |app, cx| {
        // The payload identity is only applied on success; the test asserts
        // the generation-guard rejection, so a plausible synthetic identity
        // (public fields) is enough.
        let identity = markion::DiskIdentity {
            modified: None,
            len: b"disk two\n".len() as u64,
            digest: 0xfeed,
        };
        app.apply_external_check_outcomes(
            vec![(
                ExternalCheckRequest { ..request },
                markion::ExternalCheckOutcome::Modified {
                    reload: Some(Ok(("disk two\n".to_string(), identity))),
                },
            )],
            cx,
        );
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), before_text, "newer text preserved");
        assert_eq!(tab.document.version(), before_version);
        let journal = tab.document.mutation_journal();
        let last = journal.last().expect("rejection is journaled");
        assert_eq!(last.origin, MutationOrigin::ExternalReload);
        assert_eq!(
            last.rejection,
            Some(markion::MutationRejectionReason::StaleVersion)
        );
    });
}

/// Task 4.2 — undo/redo are per-document-instance; switching tabs mid-edit
/// cannot attach one tab's history to another or leak text across slots.
#[gpui::test]
fn undo_redo_is_isolated_per_document_instance(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("# tab zero\n")),
            EditorTab::new(MarkdownDocument::from_text("# tab one\n")),
        ];
        app.view_mode = ViewMode::Edit;
        app
    });
    focus_and_park(&app, cx);

    let (id_zero, id_one) = app.update(cx, |app, _| {
        (
            app.tabs[0].document.instance_id(),
            app.tabs[1].document.instance_id(),
        )
    });
    assert_ne!(id_zero, id_one);

    // Type in tab zero, switch, type in tab one, then undo there.
    type_text(&app, cx, "zero edit ");
    app.update(cx, |app, cx| app.switch_active_tab(1, cx));
    type_text(&app, cx, "one edit ");
    cx.dispatch_action(Undo);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(app.tabs[0].document.text(), "zero edit # tab zero\n");
        assert_eq!(app.tabs[1].document.text(), "# tab one\n");
        assert_eq!(app.tabs[0].document.instance_id(), id_zero);
        assert_eq!(app.tabs[1].document.instance_id(), id_one);
        // The undo in tab one produced no mutation entry in tab zero.
        assert_eq!(app.tabs[0].document.mutation_journal_len(), 1);
        assert_eq!(
            app.tabs[1].document.mutation_journal_len(),
            2,
            "typing plus the undo restore in tab one"
        );
    });
}

/// Task 4.2 — a slow startup open must never replace whatever document took
/// over the slot it targeted; it falls back to opening a new tab.
#[gpui::test]
fn startup_open_binds_to_the_occupied_slot_generation(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("startup.md");
    fs::write(&path, "# opened file\n").unwrap();

    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("# welcome\n")),
            EditorTab::new(MarkdownDocument::from_text("# scratch\n")),
        ];
        app.active_tab = 0;
        app.view_mode = ViewMode::Edit;
        app
    });
    focus_and_park(&app, cx);

    // The read targets the welcome slot; before the background read
    // completes, the user switches to the scratch tab.
    app.update(cx, |app, cx| {
        app.apply_startup_open_intent(StartupOpenIntent::File(path.clone()), cx);
        app.switch_active_tab(1, cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(app.tabs[0].document.text(), "# welcome\n");
        assert_eq!(app.tabs[1].document.text(), "# scratch\n");
        assert_eq!(app.tabs.len(), 3, "file opens into its own tab");
        assert_eq!(app.tabs[2].document.text(), "# opened file\n");
    });
}

// ---------------------------------------------------------------------------
// State machine (tasks 4.3/4.4/5.1)
// ---------------------------------------------------------------------------

/// Per-tab reference state: the exact text every transparent operation must
/// reproduce, plus the historical bounds used by the duplication tripwire.
struct MachineTab {
    text: String,
    /// version -> text it named; a collision is the incident mechanism.
    seen_versions: std::collections::HashMap<u64, String>,
    last_version: u64,
    /// Historical heading-line count bounds observed on the document. Only
    /// heading-aware operations may raise the ceiling; runaway duplication
    /// blows through it.
    max_heading_lines: usize,
}

struct Machine {
    tabs: Vec<MachineTab>,
    active: usize,
    trace: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Input,
    ReplaceSelection,
    ExplicitRangeInput,
    StaleExplicitRangeInput,
    ImeUpdate,
    ImeCommit,
    ImeCancel,
    Enter,
    Backspace,
    Format,
    TableFormat,
    Undo,
    Redo,
    TabSwitch,
    ModeSwitch,
    DerivedRead,
    StaleCheckedMutation,
}

impl Op {
    const ALL: &[Op] = &[
        Op::Input,
        Op::ReplaceSelection,
        Op::ExplicitRangeInput,
        Op::StaleExplicitRangeInput,
        Op::ImeUpdate,
        Op::ImeCommit,
        Op::ImeCancel,
        Op::Enter,
        Op::Backspace,
        Op::Format,
        Op::TableFormat,
        Op::Undo,
        Op::Redo,
        Op::TabSwitch,
        Op::ModeSwitch,
        Op::DerivedRead,
        Op::StaleCheckedMutation,
    ];

    fn describe(self) -> &'static str {
        match self {
            Op::Input => "input",
            Op::ReplaceSelection => "replace-selection",
            Op::ExplicitRangeInput => "explicit-range-input",
            Op::StaleExplicitRangeInput => "stale-explicit-range-input",
            Op::ImeUpdate => "ime-update",
            Op::ImeCommit => "ime-commit",
            Op::ImeCancel => "ime-cancel",
            Op::Enter => "enter",
            Op::Backspace => "backspace",
            Op::Format => "format",
            Op::TableFormat => "table-format",
            Op::Undo => "undo",
            Op::Redo => "redo",
            Op::TabSwitch => "tab-switch",
            Op::ModeSwitch => "mode-switch",
            Op::DerivedRead => "derived-read",
            Op::StaleCheckedMutation => "stale-checked-mutation",
        }
    }

    /// Operations whose app result must equal the reference splice exactly.
    fn transparent(self) -> bool {
        matches!(
            self,
            Op::Input | Op::ReplaceSelection | Op::ExplicitRangeInput
        )
    }

    /// Operations that legitimately add heading lines: heading formatting,
    /// payloads that themselves contain '#', Enter continuing a heading
    /// prefix (splitting one heading line into two), and undo/redo of
    /// anything.
    fn may_raise_heading_count(self) -> bool {
        matches!(
            self,
            Op::Format
                | Op::Undo
                | Op::Redo
                | Op::Input
                | Op::ReplaceSelection
                | Op::ExplicitRangeInput
                | Op::ImeUpdate
                | Op::ImeCommit
                | Op::Enter
        )
    }
}

/// Tiny deterministic LCG so operation streams are replayable from a seed
/// without pulling in a fuzzing dependency.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 16) as usize) % bound.max(1)
    }
}

/// Caret positions stay inside the first two sections of the fixture, far
/// from the table at the tail, so transparent input never lands in a Visual
/// Edit table-cell field whose sanitization would legitimately differ from
/// a plain splice.
const SAFE_CARET_SPAN: usize = 64;

fn clamp_caret(text: &str, offset: usize) -> usize {
    let mut caret = offset.min(text.len());
    while caret < text.len() && !text.is_char_boundary(caret) {
        caret += 1;
    }
    caret
}

fn splice(text: &str, range: Range<usize>, replacement: &str) -> String {
    let mut out = String::with_capacity(text.len() + replacement.len());
    out.push_str(&text[..range.start]);
    out.push_str(replacement);
    out.push_str(&text[range.end..]);
    out
}

/// Tasks 4.3/4.4/5.1 — deterministic state machine over the §1.1–§1.9
/// heading-dense fixture with a reference oracle and fixed seeds. After
/// every step: versions are strictly monotonic per document, a version never
/// names two different texts, transparent operations match the reference
/// splice exactly, and heading lines never duplicate outside the operations
/// that may legitimately create them.
#[gpui::test]
fn mutation_state_machine_preserves_reference_and_versions(cx: &mut TestAppContext) {
    const SEEDS: &[u64] = &[0x5eed_0123, 0x5eed_0456, 0x5eed_0789];
    const STEPS: usize = 200;

    for (seed_index, &seed) in SEEDS.iter().enumerate() {
        let fixture = incident_fixture();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("machine-{seed_index}.md"));
        fs::write(&path, fixture.as_bytes()).unwrap();
        let file_document = MarkdownDocument::open(&path).unwrap();

        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![
                EditorTab::new(file_document),
                EditorTab::new(MarkdownDocument::from_text(fixture.clone())),
            ];
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        focus_and_park(&app, cx);

        let initial_headings = heading_line_count(&fixture);
        let new_tab = |text: String| MachineTab {
            text,
            seen_versions: Default::default(),
            last_version: 0,
            max_heading_lines: initial_headings,
        };
        let mut machine = Machine {
            tabs: vec![new_tab(fixture.clone()), new_tab(fixture)],
            active: 0,
            trace: Vec::new(),
        };

        let mut rng = Lcg(seed);
        for step in 0..STEPS {
            let op = Op::ALL[rng.below(Op::ALL.len())];
            let mode = app.update(cx, |app, _| app.view_mode);
            machine.trace.push(format!(
                "seed {seed_index} step {step}: {} (active {}, mode {mode:?})",
                op.describe(),
                machine.active
            ));
            run_machine_step(&app, cx, &mut machine, op, &mut rng);
            verify_and_resync(&app, cx, &mut machine, op);
        }
    }
}

fn run_machine_step(
    app: &gpui::Entity<MarkionApp>,
    cx: &mut gpui::VisualTestContext,
    machine: &mut Machine,
    op: Op,
    rng: &mut Lcg,
) {
    let active = machine.active;
    let payloads = ["a", "Z", "汉字", "🙂", "# ", "text "];
    let trace = machine.trace.last().unwrap().clone();
    match op {
        Op::Input | Op::ReplaceSelection => {
            ime_cancel(app, cx);
            let payload = payloads[rng.below(payloads.len())];
            let start = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            let end = if matches!(op, Op::Input) {
                start
            } else {
                clamp_caret(&machine.tabs[active].text, start + rng.below(12))
            };
            app.update(cx, |app, _| {
                app.active_tab_mut().selected_range = start..end;
            });
            // Sync the reference BEFORE the edit lands.
            machine.tabs[active].text =
                splice(&machine.tabs[active].text.clone(), start..end, payload);
            type_text(app, cx, payload);
        }
        Op::ExplicitRangeInput => {
            // Simulate the platform contract faithfully: query the selection
            // (refreshing the reported generation), then deliver explicit
            // offsets derived from the state the query reported.
            ime_cancel(app, cx);
            query_selection(app, cx);
            let payload = payloads[rng.below(payloads.len())];
            let (utf16_range, byte_range) = {
                let text_len = machine.tabs[active].text.len();
                let start = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
                let end = clamp_caret(&machine.tabs[active].text, start + rng.below(6));
                let _ = text_len;
                app.update(cx, |app, _| {
                    let tab = app.active_tab();
                    (
                        tab.offset_to_utf16(start)..tab.offset_to_utf16(end),
                        start..end,
                    )
                })
            };
            machine.tabs[active].text = splice(
                &machine.tabs[active].text.clone(),
                byte_range.clone(),
                payload,
            );
            type_text_in_range(app, cx, utf16_range, payload);
        }
        Op::StaleExplicitRangeInput => {
            // Capture explicit offsets from a query, then mutate the
            // document through a NON-input path (which does not refresh the
            // platform's reported generation), and submit the stale offsets:
            // they must be rejected wholesale with the document preserved.
            ime_cancel(app, cx);
            query_selection(app, cx);
            let start = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            let end = clamp_caret(&machine.tabs[active].text, start + rng.below(6));
            let utf16_range = app.update(cx, |app, _| {
                let tab = app.active_tab();
                tab.offset_to_utf16(start)..tab.offset_to_utf16(end)
            });
            // Intervening non-input mutation: a direct checked edit at the
            // caret, mirrored exactly in the reference.
            app.update(cx, |app, _| {
                app.active_tab_mut().selected_range = start..start;
            });
            let mutation = app.update(cx, |app, _| {
                let tab = app.active_tab();
                tab.document.prepare_range_mutation(
                    MutationOrigin::MarkdownFormat,
                    start..start,
                    "intervening",
                )
            });
            let receipt = app.update(cx, |app, _| {
                app.apply_document_mutation("machine-intervening", mutation)
            });
            assert!(receipt.is_some(), "{trace}: intervening edit must apply");
            machine.tabs[active].text = splice(
                &machine.tabs[active].text.clone(),
                start..start,
                "intervening",
            );

            let before = app.update(cx, |app, _| {
                let tab = app.active_tab();
                (tab.document.text().to_string(), tab.document.version())
            });
            type_text_in_range(app, cx, utf16_range, "stale");
            app.update(cx, |app, _| {
                let tab = app.active_tab();
                assert_eq!(tab.document.text(), before.0, "{trace}: text preserved");
                assert_eq!(
                    tab.document.version(),
                    before.1,
                    "{trace}: version preserved"
                );
            });
        }
        Op::ImeUpdate => {
            let composition = ["中", "中文", "中文🙂"];
            let payload = composition[rng.below(composition.len())];
            ime_mark(app, cx, payload);
        }
        Op::ImeCommit => {
            // Commit replaces the marked span with the final composition
            // text (modelled as "！").
            type_text(app, cx, "！");
        }
        Op::ImeCancel => {
            ime_cancel(app, cx);
        }
        Op::Enter => {
            ime_cancel(app, cx);
            let caret = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            app.update(cx, |app, _| {
                app.active_tab_mut().selected_range = caret..caret;
            });
            cx.dispatch_action(InsertNewline);
            cx.run_until_parked();
        }
        Op::Backspace => {
            ime_cancel(app, cx);
            let caret = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            app.update(cx, |app, _| {
                app.active_tab_mut().selected_range = caret..caret;
            });
            cx.dispatch_action(Backspace);
            cx.run_until_parked();
        }
        Op::Format => {
            ime_cancel(app, cx);
            let start = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            let end = clamp_caret(&machine.tabs[active].text, start + rng.below(20));
            let use_heading = rng.below(2) == 0;
            app.update(cx, |app, cx| {
                app.active_tab_mut().selected_range = start..end;
                let format = if use_heading {
                    MarkdownFormat::Heading(2)
                } else {
                    MarkdownFormat::Bold
                };
                app.apply_markdown_format(format, "format".into(), cx);
            });
            cx.run_until_parked();
        }
        Op::TableFormat => {
            // Aim at the fixture's table row if it still exists.
            let offset = machine.tabs[active]
                .text
                .find("| 1 | 2 |")
                .map(|at| at + 2)
                .unwrap_or(0);
            app.update(cx, |app, cx| {
                app.apply_table_edit_at(offset, TableEdit::Format, "table".into(), cx);
            });
            cx.run_until_parked();
        }
        Op::Undo | Op::Redo => {
            let is_undo = matches!(op, Op::Undo);
            if is_undo {
                cx.dispatch_action(Undo);
            } else {
                cx.dispatch_action(Redo);
            }
            cx.run_until_parked();
        }
        Op::TabSwitch => {
            let target = rng.below(machine.tabs.len());
            app.update(cx, |app, cx| app.switch_active_tab(target, cx));
            machine.active = target;
        }
        Op::ModeSwitch => {
            app.update(cx, |app, _| {
                app.view_mode = if matches!(app.view_mode, ViewMode::VisualEdit) {
                    ViewMode::Edit
                } else {
                    ViewMode::VisualEdit
                };
            });
            cx.run_until_parked();
        }
        Op::DerivedRead => {
            app.update(cx, |app, _| {
                let tab = app.active_tab();
                let _ = tab.document.visual_blocks_shared();
                let _ = tab.document.preview_blocks_shared();
                let _ = tab.document.outline();
                let _ = tab.shared_document_text();
            });
        }
        Op::StaleCheckedMutation => {
            // Prepare a checked mutation, mutate past it, then apply: the
            // boundary must reject and preserve everything.
            let caret = clamp_caret(&machine.tabs[active].text, rng.below(SAFE_CARET_SPAN));
            let prepared = app.update(cx, |app, _| {
                app.active_tab().document.prepare_range_mutation(
                    MutationOrigin::ExactBlockEdit,
                    caret..caret,
                    "stale",
                )
            });
            type_text(app, cx, "moved-on");
            let before = app.update(cx, |app, _| {
                let tab = app.active_tab();
                (
                    tab.document.text().to_string(),
                    tab.document.version(),
                    tab.document.is_dirty(),
                )
            });
            let outcome = app.update(cx, |app, _| {
                app.active_tab_mut()
                    .document
                    .apply_checked_mutation(prepared)
            });
            assert!(outcome.is_err(), "{trace}: stale mutation must be rejected");
            app.update(cx, |app, _| {
                let tab = app.active_tab();
                assert_eq!(tab.document.text(), before.0, "{trace}: text preserved");
                assert_eq!(
                    tab.document.version(),
                    before.1,
                    "{trace}: version preserved"
                );
                assert_eq!(tab.document.is_dirty(), before.2);
            });
            // Reference absorbs the intervening edit at the caret.
            machine.tabs[active].text =
                splice(&machine.tabs[active].text.clone(), caret..caret, "moved-on");
        }
    }
}

/// Post-step contract check, then resynchronize the reference with the app
/// state for the operations whose Markdown semantics the harness does not
/// mirror (the generic invariants still hold for every one of them).
fn verify_and_resync(
    app: &gpui::Entity<MarkionApp>,
    cx: &mut gpui::VisualTestContext,
    machine: &mut Machine,
    op: Op,
) {
    let trace = machine.trace.last().unwrap().clone();
    let app_states: Vec<(u64, String)> = app.update(cx, |app, _| {
        app.tabs
            .iter()
            .map(|tab| {
                let doc = tab.document_tab().unwrap();
                (doc.document.version(), doc.document.text().to_string())
            })
            .collect()
    });
    assert_eq!(
        app_states.len(),
        machine.tabs.len(),
        "{trace}: tab count changed unexpectedly"
    );

    for index in 0..machine.tabs.len() {
        let (version, text) = (app_states[index].0, app_states[index].1.clone());
        let machine_tab = &mut machine.tabs[index];

        // Versions never go backwards per document: an undo restore that
        // resurrects an older counter is the incident mechanism. A step that
        // does not touch the tab may leave the version unchanged.
        assert!(
            version >= machine_tab.last_version,
            "{trace}: tab {index} version went backwards ({version} < {})",
            machine_tab.last_version
        );

        // A version never names two different texts.
        if let Some(previous) = machine_tab.seen_versions.insert(version, text.clone()) {
            assert_eq!(
                previous, text,
                "{trace}: tab {index} version {version} named two different texts"
            );
        }
        machine_tab.last_version = version;

        // Transparent operations must equal the reference splice exactly.
        if op.transparent() && index == machine.active {
            assert_eq!(
                text, machine_tab.text,
                "{trace}: tab {index} diverged from the reference splice"
            );
        }

        // Heading duplication tripwire: heading lines may only grow through
        // heading-capable operations; runaway duplication blows through the
        // historical ceiling.
        let headings = heading_line_count(&text);
        if !op.may_raise_heading_count() {
            assert!(
                headings <= machine_tab.max_heading_lines,
                "{trace}: tab {index} heading lines grew to {headings} past the \
                 historical ceiling {} without a heading-capable operation — \
                 duplication signature",
                machine_tab.max_heading_lines
            );
        }
        machine_tab.max_heading_lines = machine_tab.max_heading_lines.max(headings);

        // Resync the reference with the authoritative app state.
        machine_tab.text = text;
    }
}
