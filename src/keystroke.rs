//! Helpers for GPUI keystroke strings (`"secondary-shift-s"`).
//!
//! Shared by the shortcut registry, menu shortcut labels, and the
//! Preferences-panel shortcut editor so default and overridden bindings are
//! validated, compared, and rendered through one code path. Kept free of GPUI
//! dependencies so `model`/`storage` layering stays intact.

use crate::i18n::ShortcutPlatform;

/// A keystroke string decomposed into modifier flags and a key name. Modifier
/// aliases (`secondary`, `option`, `super`, ...) are resolved so two strings
/// that mean the same keystroke compare equal.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeystrokeParts {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Cmd on macOS, the Windows/Super key elsewhere.
    pub platform: bool,
    pub function: bool,
    /// The non-modifier key, lowercased.
    pub key: KeyName,
}

/// The non-modifier part of a keystroke, lowercased.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyName(pub String);

impl KeystrokeParts {
    /// Parse a GPUI keystroke string into canonical parts. `secondary` is
    /// resolved against `platform` so a stored override and a default binding
    /// written with different aliases still compare equal. Returns `None`
    /// when the string has no key part or contains an unknown modifier.
    pub fn parse(binding: &str, platform: ShortcutPlatform) -> Option<Self> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut platform_mod = false;
        let mut function = false;
        let segments: Vec<&str> = binding.split('-').collect();
        let (key_segments, mods) = segments.split_last()?;
        let key = key_segments.trim().to_ascii_lowercase();
        if key.is_empty() {
            return None;
        }
        for raw in mods {
            match raw.trim().to_ascii_lowercase().as_str() {
                "ctrl" | "control" => ctrl = true,
                "alt" | "option" => alt = true,
                "shift" => shift = true,
                "cmd" | "command" | "super" | "win" => platform_mod = true,
                "fn" | "function" => function = true,
                "secondary" => match platform {
                    ShortcutPlatform::MacOS => platform_mod = true,
                    ShortcutPlatform::WindowsLinux => ctrl = true,
                },
                _ => return None,
            }
        }
        Some(Self {
            ctrl,
            alt,
            shift,
            platform: platform_mod,
            function,
            key: KeyName(key),
        })
    }

    /// Whether the keystroke may be assigned to a menu action: it must carry
    /// a non-shift modifier (so plain typing is never hijacked) unless the
    /// key itself is a function key (F1-F12).
    pub fn is_assignable(&self) -> bool {
        if self.ctrl || self.alt || self.platform {
            return true;
        }
        is_function_key(&self.key.0)
    }
}

fn is_function_key(key: &str) -> bool {
    let Some(number) = key.strip_prefix('f') else {
        return false;
    };
    matches!(number.parse::<u8>(), Ok(1..=12))
}

/// Render a keystroke string as a user-facing label for `platform`
/// (`"secondary-shift-s"` -> `"Ctrl+Shift+S"` on Windows/Linux,
/// `"Cmd+Shift+S"` on macOS). Unparseable strings are returned unchanged so a
/// bad stored value never renders as an empty label.
pub fn format_keystroke_label(binding: &str, platform: ShortcutPlatform) -> String {
    let Some(parts) = KeystrokeParts::parse(binding, platform) else {
        return binding.to_string();
    };
    let macos = platform == ShortcutPlatform::MacOS;
    let mut segments: Vec<&str> = Vec::new();
    if parts.ctrl {
        segments.push("Ctrl");
    }
    if parts.platform {
        segments.push(if macos { "Cmd" } else { "Win" });
    }
    if parts.alt {
        segments.push(if macos { "Option" } else { "Alt" });
    }
    if parts.shift {
        segments.push("Shift");
    }
    if parts.function {
        segments.push("Fn");
    }
    let key = display_key(&parts.key.0);
    let mut label = segments.join("+");
    if !label.is_empty() {
        label.push('+');
    }
    label.push_str(&key);
    label
}

/// User-facing rendering for named keys; single characters are uppercased.
fn display_key(key: &str) -> String {
    let named = match key {
        "comma" => Some(","),
        "period" => Some("."),
        "slash" => Some("/"),
        "backslash" => Some("\\"),
        "semicolon" => Some(";"),
        "quote" => Some("'"),
        "backquote" | "grave" => Some("`"),
        "minus" => Some("-"),
        "equal" => Some("="),
        "leftbracket" | "bracketleft" => Some("["),
        "rightbracket" | "bracketright" => Some("]"),
        _ => None,
    };
    if let Some(symbol) = named {
        return symbol.to_string();
    }
    if key.chars().count() == 1 {
        return key.to_uppercase();
    }
    // Capitalize multi-character key names: "enter" -> "Enter", "f6" -> "F6".
    let mut chars = key.chars();
    let mut rendered = String::new();
    if let Some(first) = chars.next() {
        rendered.extend(first.to_uppercase());
    }
    rendered.push_str(chars.as_str());
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_secondary_per_platform() {
        assert_eq!(
            format_keystroke_label("secondary-shift-s", ShortcutPlatform::WindowsLinux),
            "Ctrl+Shift+S"
        );
        assert_eq!(
            format_keystroke_label("secondary-shift-s", ShortcutPlatform::MacOS),
            "Cmd+Shift+S"
        );
    }

    #[test]
    fn formats_named_and_function_keys() {
        assert_eq!(
            format_keystroke_label("secondary-comma", ShortcutPlatform::WindowsLinux),
            "Ctrl+,"
        );
        assert_eq!(format_keystroke_label("f6", ShortcutPlatform::MacOS), "F6");
        assert_eq!(
            format_keystroke_label("secondary-alt-enter", ShortcutPlatform::MacOS),
            "Cmd+Option+Enter"
        );
        assert_eq!(
            format_keystroke_label("shift-f3", ShortcutPlatform::WindowsLinux),
            "Shift+F3"
        );
    }

    #[test]
    fn parses_aliases_to_equal_parts() {
        let secondary =
            KeystrokeParts::parse("secondary-b", ShortcutPlatform::WindowsLinux).unwrap();
        let ctrl = KeystrokeParts::parse("ctrl-b", ShortcutPlatform::WindowsLinux).unwrap();
        assert_eq!(secondary, ctrl);
        let cmd = KeystrokeParts::parse("cmd-b", ShortcutPlatform::MacOS).unwrap();
        let secondary_mac = KeystrokeParts::parse("secondary-b", ShortcutPlatform::MacOS).unwrap();
        assert_eq!(cmd, secondary_mac);
        assert_ne!(secondary, secondary_mac);
    }

    #[test]
    fn assignability_rules() {
        let win = ShortcutPlatform::WindowsLinux;
        assert!(
            KeystrokeParts::parse("ctrl-n", win)
                .unwrap()
                .is_assignable()
        );
        assert!(KeystrokeParts::parse("f1", win).unwrap().is_assignable());
        assert!(
            KeystrokeParts::parse("alt-f4", win)
                .unwrap()
                .is_assignable()
        );
        assert!(!KeystrokeParts::parse("n", win).unwrap().is_assignable());
        // Shift alone must not count: shift-a types a capital letter.
        assert!(
            !KeystrokeParts::parse("shift-a", win)
                .unwrap()
                .is_assignable()
        );
        // f13+ are not in the supported function-key range.
        assert!(!KeystrokeParts::parse("f13", win).unwrap().is_assignable());
    }

    #[test]
    fn rejects_unknown_modifiers_and_missing_key() {
        assert!(KeystrokeParts::parse("bogus-x", ShortcutPlatform::WindowsLinux).is_none());
        assert!(KeystrokeParts::parse("", ShortcutPlatform::WindowsLinux).is_none());
    }

    #[test]
    fn unparseable_label_falls_back_to_raw_string() {
        assert_eq!(
            format_keystroke_label("bogus-x", ShortcutPlatform::WindowsLinux),
            "bogus-x"
        );
    }
}
