//! SettingValue enum for storing different types of setting values

use serde::{Deserialize, Serialize};

/// Represents different types of setting values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SettingValue {
    /// String value
    String(String),
    /// Integer value (i64)
    Integer(i64),
    /// Floating point value (f64)
    Float(f64),
    /// Boolean value
    Boolean(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_value_partial_eq() {
        let val1 = SettingValue::String("test".to_string());
        let val2 = SettingValue::String("test".to_string());
        assert_eq!(val1, val2);
    }
}
