//! Unit tests for SettingValue enum
//!
//! Tests serialization, deserialization, and type conversions

use settings_manager::SettingValue;

#[test]
fn test_setting_value_string_variant() {
    // Given a string setting value
    let value = SettingValue::String("test".to_string());

    // When converting to string
    // Then it should match the original
    match value {
        SettingValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_setting_value_integer_variant() {
    // Given an integer setting value
    let value = SettingValue::Integer(42);

    // When pattern matching
    // Then it should have the correct value
    match value {
        SettingValue::Integer(i) => assert_eq!(i, 42),
        _ => panic!("Expected Integer variant"),
    }
}

#[test]
fn test_setting_value_float_variant() {
    // Given a float setting value
    let value = SettingValue::Float(3.14);

    // When pattern matching
    // Then it should have the correct value
    match value {
        SettingValue::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected Float variant"),
    }
}

#[test]
fn test_setting_value_boolean_variant() {
    // Given a boolean setting value
    let value = SettingValue::Boolean(true);

    // When pattern matching
    // Then it should have the correct value
    match value {
        SettingValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_setting_value_serialization() {
    // Given various setting values
    let string_val = SettingValue::String("theme".to_string());
    let int_val = SettingValue::Integer(1024);
    let float_val = SettingValue::Float(60.0);
    let bool_val = SettingValue::Boolean(true);

    // When serializing to YAML
    let string_yaml = serde_yaml::to_string(&string_val).unwrap();
    let int_yaml = serde_yaml::to_string(&int_val).unwrap();
    let float_yaml = serde_yaml::to_string(&float_val).unwrap();
    let bool_yaml = serde_yaml::to_string(&bool_val).unwrap();

    // Then the YAML should contain the values
    assert!(string_yaml.contains("String"));
    assert!(int_yaml.contains("Integer"));
    assert!(float_yaml.contains("Float"));
    assert!(bool_yaml.contains("Boolean"));
}

#[test]
fn test_setting_value_deserialization() {
    // Given YAML representations
    let yaml_str = "!String theme";
    let yaml_int = "!Integer 1024";
    let yaml_float = "!Float 60.0";
    let yaml_bool = "!Boolean true";

    // When deserializing
    let string_val: SettingValue = serde_yaml::from_str(yaml_str).unwrap();
    let int_val: SettingValue = serde_yaml::from_str(yaml_int).unwrap();
    let float_val: SettingValue = serde_yaml::from_str(yaml_float).unwrap();
    let bool_val: SettingValue = serde_yaml::from_str(yaml_bool).unwrap();

    // Then the values should be correct
    match string_val {
        SettingValue::String(s) => assert_eq!(s, "theme"),
        _ => panic!("Expected String variant"),
    }
    match int_val {
        SettingValue::Integer(i) => assert_eq!(i, 1024),
        _ => panic!("Expected Integer variant"),
    }
    match float_val {
        SettingValue::Float(f) => assert!((f - 60.0).abs() < 0.001),
        _ => panic!("Expected Float variant"),
    }
    match bool_val {
        SettingValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_setting_value_clone() {
    // Given a setting value
    let original = SettingValue::String("test".to_string());

    // When cloning
    let cloned = original.clone();

    // Then both should be equal
    match (&original, &cloned) {
        (SettingValue::String(s1), SettingValue::String(s2)) => assert_eq!(s1, s2),
        _ => panic!("Clone should preserve type and value"),
    }
}

#[test]
fn test_setting_value_debug() {
    // Given a setting value
    let value = SettingValue::String("debug_test".to_string());

    // When formatting with debug
    let debug_str = format!("{:?}", value);

    // Then it should contain useful information
    assert!(debug_str.contains("String"));
    assert!(debug_str.contains("debug_test"));
}
