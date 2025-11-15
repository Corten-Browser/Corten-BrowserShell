use shared_types::*;

#[test]
fn test_keyboard_shortcut_variants() {
    // Test all variants exist
    let shortcuts = vec![
        KeyboardShortcut::CtrlT,
        KeyboardShortcut::CtrlW,
        KeyboardShortcut::CtrlN,
        KeyboardShortcut::CtrlShiftT,
        KeyboardShortcut::CtrlL,
        KeyboardShortcut::F5,
        KeyboardShortcut::CtrlR,
        KeyboardShortcut::CtrlShiftR,
    ];

    assert_eq!(shortcuts.len(), 8);
}

#[test]
fn test_keyboard_shortcut_equality() {
    assert_eq!(KeyboardShortcut::CtrlT, KeyboardShortcut::CtrlT);
    assert_ne!(KeyboardShortcut::CtrlT, KeyboardShortcut::CtrlW);
}

#[test]
fn test_keyboard_shortcut_clone() {
    let shortcut1 = KeyboardShortcut::CtrlShiftT;
    let shortcut2 = shortcut1.clone();

    assert_eq!(shortcut1, shortcut2);
}

#[test]
fn test_keyboard_shortcut_serialization() {
    let shortcut = KeyboardShortcut::CtrlT;

    // Serialize to JSON
    let json = serde_json::to_string(&shortcut).expect("Failed to serialize");

    // Deserialize back
    let deserialized: KeyboardShortcut =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(shortcut, deserialized);
}

#[test]
fn test_all_keyboard_shortcuts_serializable() {
    let shortcuts = vec![
        KeyboardShortcut::CtrlT,
        KeyboardShortcut::CtrlW,
        KeyboardShortcut::CtrlN,
        KeyboardShortcut::CtrlShiftT,
        KeyboardShortcut::CtrlL,
        KeyboardShortcut::F5,
        KeyboardShortcut::CtrlR,
        KeyboardShortcut::CtrlShiftR,
    ];

    for shortcut in shortcuts {
        let json = serde_json::to_string(&shortcut).expect("Failed to serialize");
        let _deserialized: KeyboardShortcut =
            serde_json::from_str(&json).expect("Failed to deserialize");
    }
}

#[test]
fn test_keyboard_shortcut_debug() {
    let shortcut = KeyboardShortcut::CtrlT;
    let debug_str = format!("{:?}", shortcut);

    assert!(debug_str.contains("CtrlT"));
}
