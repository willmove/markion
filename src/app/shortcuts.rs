//! Customizable menu-action shortcuts: override sanitizing, capture-based
//! reassignment with conflict detection, per-action reset, and live
//! rebinding. The registry itself lives in `mod.rs` (`menu_shortcuts`), the
//! binding code path in `bootstrap.rs` (`bind_app_keys`).

use super::*;

use markion::keystroke::{KeystrokeParts, format_keystroke_label};

/// Fixed bindings that are never customizable; a captured keystroke matching
/// any of these is rejected just like a conflict with a registry action, so
/// typing, navigation, and file-tree keys can never be hijacked.
const RESERVED_BINDINGS: &[&str] = &[
    "backspace",
    "delete",
    "left",
    "right",
    "up",
    "down",
    "shift-left",
    "shift-right",
    "shift-up",
    "shift-down",
    "home",
    "end",
    "enter",
    "tab",
    "shift-tab",
    "escape",
    "f5",
    "secondary-alt-n",
    "secondary-alt-shift-n",
    "f2",
    "secondary-delete",
    "ctrl-shift-alt-m",
];

fn bindings_equivalent(left: &str, right: &str, platform: ShortcutPlatform) -> bool {
    match (
        KeystrokeParts::parse(left, platform),
        KeystrokeParts::parse(right, platform),
    ) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

/// Drop stored overrides that cannot take effect — unknown action ids or
/// keystroke strings that fail validation — logging each dropped entry.
pub(super) fn sanitized_shortcut_overrides(
    overrides: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    overrides
        .iter()
        .filter(|(id, binding)| {
            let known = shortcut_by_id(id).is_some();
            let parseable = KeystrokeParts::parse(binding, ShortcutPlatform::current()).is_some()
                && gpui::Keystroke::parse(binding).is_ok();
            if !(known && parseable) {
                tracing::warn!(
                    action = %id,
                    binding = %binding,
                    "ignoring invalid shortcut override"
                );
            }
            known && parseable
        })
        .map(|(id, binding)| (id.clone(), binding.clone()))
        .collect()
}

/// Serialize a captured key event into a GPUI binding string. Modifier-only
/// presses return `None` so the user can keep holding keys to compose a
/// combination.
fn binding_from_capture(event: &KeyDownEvent) -> Option<String> {
    let keystroke = &event.keystroke;
    // Ignore presses of the modifier keys themselves.
    if matches!(
        keystroke.key.as_str(),
        "control" | "shift" | "alt" | "meta" | "super" | "fn" | "capslock" | "numlock"
    ) {
        return None;
    }
    let modifiers = keystroke.modifiers;
    let mut binding = String::new();
    if modifiers.control {
        binding.push_str("ctrl-");
    }
    if modifiers.platform {
        binding.push_str(if cfg!(target_os = "macos") {
            "cmd-"
        } else {
            "win-"
        });
    }
    if modifiers.alt {
        binding.push_str("alt-");
    }
    if modifiers.shift {
        binding.push_str("shift-");
    }
    if modifiers.function {
        binding.push_str("fn-");
    }
    binding.push_str(&keystroke.key);
    Some(binding)
}

impl MarkionApp {
    /// Rebuild the whole keymap from the current overrides. Used after any
    /// shortcut change and when capture mode ends.
    pub(super) fn rebind_keys(&self, cx: &mut Context<Self>) {
        cx.clear_key_bindings();
        bind_app_keys(cx, &self.shortcut_overrides);
    }

    /// The effective label for a registry action (curated default or
    /// formatted override), used by menu rows and the shortcut reference.
    pub(super) fn shortcut_label(
        &self,
        shortcut: &MenuShortcut,
        platform: ShortcutPlatform,
    ) -> String {
        shortcut.effective_label(&self.shortcut_overrides, platform)
    }

    /// Localized catalog label of the row containing `action_id`, for
    /// conflict feedback.
    fn shortcut_action_name(&self, action_id: &str) -> Option<String> {
        let catalog = shortcut_catalog(self.language, self.heading_menu_max_level);
        catalog
            .sections
            .iter()
            .flat_map(|section| section.actions.iter())
            .find(|action| action.ids().contains(&action_id))
            .map(|action| action.label.to_string())
    }

    /// Rejection reason when `binding` may not be assigned to `action_id`.
    pub(super) fn shortcut_assignment_error(
        &self,
        action_id: &str,
        binding: &str,
    ) -> Option<ShortcutCaptureError> {
        let platform = ShortcutPlatform::current();
        let Some(parts) = KeystrokeParts::parse(binding, platform) else {
            return Some(ShortcutCaptureError::NotAssignable);
        };
        if gpui::Keystroke::parse(binding).is_err() {
            return Some(ShortcutCaptureError::NotAssignable);
        }
        for reserved in RESERVED_BINDINGS {
            if KeystrokeParts::parse(reserved, platform).is_some_and(|candidate| candidate == parts)
            {
                return Some(ShortcutCaptureError::Conflict(format_keystroke_label(
                    reserved, platform,
                )));
            }
        }
        if !parts.is_assignable() {
            return Some(ShortcutCaptureError::NotAssignable);
        }
        // Same binding as the action already has: not an error, no change.
        for shortcut in menu_shortcuts::ALL {
            if shortcut.id == action_id {
                continue;
            }
            let Some(other) = shortcut.effective_binding(&self.shortcut_overrides) else {
                continue;
            };
            if KeystrokeParts::parse(other, platform).is_some_and(|candidate| candidate == parts) {
                let name = self
                    .shortcut_action_name(shortcut.id)
                    .unwrap_or_else(|| shortcut.id.to_string());
                return Some(ShortcutCaptureError::Conflict(name));
            }
        }
        None
    }

    /// Enter capture mode for a shortcut row. The application keymap is
    /// cleared while capturing so the pressed combination cannot dispatch an
    /// action; every exit path calls `rebind_keys`.
    pub(super) fn begin_shortcut_capture(
        &mut self,
        action_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if shortcut_by_id(action_id).is_none() {
            return;
        }
        self.shortcut_capture = Some(ShortcutCapture {
            action_id: action_id.to_string(),
            error: None,
        });
        cx.clear_key_bindings();
        window.focus(&self.preferences_panel_focus);
        cx.notify();
    }

    /// Leave capture mode without changing anything and restore the keymap.
    pub(super) fn cancel_shortcut_capture(&mut self, cx: &mut Context<Self>) {
        if self.shortcut_capture.take().is_some() {
            self.rebind_keys(cx);
            cx.notify();
        }
    }

    /// Handle a key press while a row is capturing.
    pub(super) fn handle_shortcut_capture_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(capture) = self.shortcut_capture.clone() else {
            return;
        };
        if event.keystroke.key == "escape" {
            self.cancel_shortcut_capture(cx);
            return;
        }
        let Some(binding) = binding_from_capture(event) else {
            // Modifier-only press: keep waiting for the full combination.
            return;
        };
        let platform = ShortcutPlatform::current();
        if shortcut_by_id(&capture.action_id).is_some_and(|shortcut| {
            shortcut
                .effective_binding(&self.shortcut_overrides)
                .is_some_and(|current| bindings_equivalent(&binding, current, platform))
        }) {
            // Pressed the binding it already has: just close capture.
            self.cancel_shortcut_capture(cx);
            return;
        }
        match self.shortcut_assignment_error(&capture.action_id, &binding) {
            Some(error) => {
                if let Some(capture) = self.shortcut_capture.as_mut() {
                    capture.error = Some(error);
                }
                cx.notify();
            }
            None => {
                self.shortcut_capture = None;
                if shortcut_by_id(&capture.action_id).is_some_and(|shortcut| {
                    shortcut
                        .binding
                        .is_some_and(|default| bindings_equivalent(&binding, default, platform))
                }) {
                    // Same as the default: store nothing, drop any override.
                    self.shortcut_overrides.remove(&capture.action_id);
                } else {
                    self.shortcut_overrides
                        .insert(capture.action_id.clone(), binding.clone());
                }
                self.persist_preferences();
                self.rebind_keys(cx);
                self.status = self.trf(
                    Msg::StatusShortcutUpdated,
                    &[&format_keystroke_label(
                        &binding,
                        ShortcutPlatform::current(),
                    )],
                );
                window.focus(&self.preferences_panel_focus);
                cx.notify();
            }
        }
    }

    /// Restore a single action to its default binding.
    pub(super) fn reset_shortcut(&mut self, action_id: &str, cx: &mut Context<Self>) {
        let capture_was_active = self.shortcut_capture.take().is_some();
        if self.shortcut_overrides.remove(action_id).is_none() && !capture_was_active {
            return;
        }
        self.persist_preferences();
        self.rebind_keys(cx);
        self.status = t(self.language, Msg::StatusShortcutReset).into();
        cx.notify();
    }

    /// Clear every shortcut override (preferences reset path).
    pub(super) fn clear_shortcut_overrides(&mut self, cx: &mut Context<Self>) {
        let capture_was_active = self.shortcut_capture.take().is_some();
        if self.shortcut_overrides.is_empty() && !capture_was_active {
            return;
        }
        self.shortcut_overrides.clear();
        self.rebind_keys(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_equivalence_resolves_secondary_per_platform() {
        assert!(bindings_equivalent(
            "secondary-b",
            "ctrl-b",
            ShortcutPlatform::WindowsLinux,
        ));
        assert!(bindings_equivalent(
            "secondary-b",
            "cmd-b",
            ShortcutPlatform::MacOS,
        ));
        assert!(!bindings_equivalent(
            "secondary-b",
            "ctrl-b",
            ShortcutPlatform::MacOS,
        ));
    }

    #[test]
    fn sanitize_drops_unknown_ids_and_bad_keystrokes() {
        let mut overrides = BTreeMap::new();
        overrides.insert("bold".to_string(), "ctrl-alt-b".to_string());
        overrides.insert("no-such-action".to_string(), "ctrl-1".to_string());
        overrides.insert("italic".to_string(), "bogus-mod-x".to_string());
        let clean = sanitized_shortcut_overrides(&overrides);
        assert_eq!(clean.len(), 1);
        assert_eq!(clean.get("bold").map(String::as_str), Some("ctrl-alt-b"));
    }

    #[test]
    fn reserved_bindings_are_not_registry_defaults() {
        // Every reserved binding must stay out of the registry defaults, or a
        // default shortcut would be flagged as conflicting with itself.
        for shortcut in menu_shortcuts::ALL {
            let Some(binding) = shortcut.binding else {
                continue;
            };
            assert!(
                !RESERVED_BINDINGS.contains(&binding),
                "{} is both reserved and a registry default",
                shortcut.id
            );
        }
    }
}
