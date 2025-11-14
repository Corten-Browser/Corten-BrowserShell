//! Default settings for the browser shell
//!
//! Based on Appendix B: Configuration Schema from the specification

use crate::SettingValue;
use std::collections::HashMap;

/// Creates the default settings map based on the specification
pub fn create_defaults() -> HashMap<String, SettingValue> {
    let mut defaults = HashMap::new();

    // Window settings (from spec lines 1293-1299)
    defaults.insert(
        "window.default_width".to_string(),
        SettingValue::Integer(1024),
    );
    defaults.insert(
        "window.default_height".to_string(),
        SettingValue::Integer(768),
    );
    defaults.insert("window.min_width".to_string(), SettingValue::Integer(400));
    defaults.insert("window.min_height".to_string(), SettingValue::Integer(300));
    defaults.insert(
        "window.allow_resize".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "window.start_maximized".to_string(),
        SettingValue::Boolean(false),
    );

    // Tab settings (from spec lines 1301-1306)
    defaults.insert(
        "tabs.enable_process_isolation".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert("tabs.max_processes".to_string(), SettingValue::Integer(50));
    defaults.insert(
        "tabs.recycle_after_navigations".to_string(),
        SettingValue::Integer(5),
    );
    defaults.insert(
        "tabs.restore_on_crash".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert("tabs.lazy_loading".to_string(), SettingValue::Boolean(true));

    // UI settings (from spec lines 1308-1313)
    defaults.insert(
        "ui.theme".to_string(),
        SettingValue::String("light".to_string()),
    );
    defaults.insert(
        "ui.show_bookmarks_bar".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "ui.show_status_bar".to_string(),
        SettingValue::Boolean(false),
    );
    defaults.insert(
        "ui.animations_enabled".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert("ui.font_size".to_string(), SettingValue::Integer(14));

    // Performance settings (from spec lines 1315-1319)
    defaults.insert(
        "performance.render_fps".to_string(),
        SettingValue::Integer(60),
    );
    defaults.insert(
        "performance.max_message_queue".to_string(),
        SettingValue::Integer(10000),
    );
    defaults.insert(
        "performance.compositor_threads".to_string(),
        SettingValue::Integer(4),
    );
    defaults.insert(
        "performance.raster_threads".to_string(),
        SettingValue::Integer(4),
    );

    // Security settings (from spec lines 1321-1327)
    defaults.insert(
        "security.enable_sandbox".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "security.allow_javascript".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "security.allow_plugins".to_string(),
        SettingValue::Boolean(false),
    );
    defaults.insert(
        "security.block_third_party_cookies".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "security.enable_webrtc".to_string(),
        SettingValue::Boolean(false),
    );

    // Network settings (from spec lines 1329-1333)
    defaults.insert(
        "network.max_connections_per_host".to_string(),
        SettingValue::Integer(6),
    );
    defaults.insert(
        "network.connection_timeout".to_string(),
        SettingValue::Integer(30),
    );
    defaults.insert(
        "network.enable_http2".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "network.enable_quic".to_string(),
        SettingValue::Boolean(true),
    );

    // Privacy settings (from spec lines 1335-1338)
    defaults.insert(
        "privacy.do_not_track".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "privacy.clear_on_exit".to_string(),
        SettingValue::Boolean(false),
    );
    defaults.insert(
        "privacy.private_browsing_available".to_string(),
        SettingValue::Boolean(true),
    );

    // Developer settings (from spec lines 1340-1343)
    defaults.insert(
        "developer.enable_devtools".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "developer.enable_extensions".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "developer.allow_experimental_features".to_string(),
        SettingValue::Boolean(false),
    );

    defaults
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_not_empty() {
        let defaults = create_defaults();
        assert!(!defaults.is_empty());
    }

    #[test]
    fn test_defaults_window_settings() {
        let defaults = create_defaults();
        assert_eq!(
            defaults.get("window.default_width"),
            Some(&SettingValue::Integer(1024))
        );
    }
}
