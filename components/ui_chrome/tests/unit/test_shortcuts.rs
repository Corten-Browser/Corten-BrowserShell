// @validates: REQ-UI-006
//! Unit tests for Keyboard shortcuts handler

use ui_chrome::shortcuts::{ShortcutHandler, ShortcutAction, KeyModifiers};

#[test]
fn test_shortcut_handler_creation() {
    let handler = ShortcutHandler::new();
    assert_eq!(handler.shortcut_count(), 0);
}

#[test]
fn test_shortcut_register() {
    let mut handler = ShortcutHandler::new();
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    handler.register("T".to_string(), modifiers, ShortcutAction::NewTab);
    assert_eq!(handler.shortcut_count(), 1);
}

#[test]
fn test_shortcut_match_exact() {
    let mut handler = ShortcutHandler::new();
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    handler.register("N".to_string(), modifiers, ShortcutAction::NewWindow);

    let test_modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    let action = handler.match_shortcut("N", &test_modifiers);
    assert_eq!(action, Some(ShortcutAction::NewWindow));
}

#[test]
fn test_shortcut_no_match_wrong_key() {
    let mut handler = ShortcutHandler::new();
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    handler.register("N".to_string(), modifiers, ShortcutAction::NewWindow);

    let test_modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    let action = handler.match_shortcut("T", &test_modifiers);
    assert_eq!(action, None);
}

#[test]
fn test_shortcut_no_match_wrong_modifiers() {
    let mut handler = ShortcutHandler::new();
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    handler.register("N".to_string(), modifiers, ShortcutAction::NewWindow);

    let test_modifiers = KeyModifiers {
        ctrl: false,
        alt: true,
        shift: false,
        meta: false,
    };

    let action = handler.match_shortcut("N", &test_modifiers);
    assert_eq!(action, None);
}

#[test]
fn test_shortcut_unregister() {
    let mut handler = ShortcutHandler::new();
    let modifiers = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };

    handler.register("W".to_string(), modifiers.clone(), ShortcutAction::CloseTab);
    assert_eq!(handler.shortcut_count(), 1);

    let result = handler.unregister("W", &modifiers);
    assert!(result.is_ok());
    assert_eq!(handler.shortcut_count(), 0);
}

#[test]
fn test_shortcut_get_all() {
    let mut handler = ShortcutHandler::new();
    let modifiers1 = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };
    let modifiers2 = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: true,
        meta: false,
    };

    handler.register("N".to_string(), modifiers1, ShortcutAction::NewWindow);
    handler.register("T".to_string(), modifiers2, ShortcutAction::NewTab);

    let all = handler.get_all_shortcuts();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_key_modifiers_equality() {
    let mod1 = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };
    let mod2 = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };
    let mod3 = KeyModifiers {
        ctrl: false,
        alt: true,
        shift: false,
        meta: false,
    };

    assert_eq!(mod1, mod2);
    assert_ne!(mod1, mod3);
}

#[test]
fn test_shortcut_action_variants() {
    assert_eq!(ShortcutAction::NewTab, ShortcutAction::NewTab);
    assert_ne!(ShortcutAction::NewTab, ShortcutAction::NewWindow);
}
