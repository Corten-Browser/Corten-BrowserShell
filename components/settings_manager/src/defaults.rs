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
    // HTTP/3 support (uses QUIC as transport layer)
    defaults.insert(
        "network.enable_http3".to_string(),
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

    // General settings
    defaults.insert(
        "general.home_page".to_string(),
        SettingValue::String("about:blank".to_string()),
    );
    defaults.insert(
        "general.startup_behavior".to_string(),
        SettingValue::String("new_tab".to_string()), // new_tab, home_page, restore_session
    );
    defaults.insert(
        "general.default_search_engine".to_string(),
        SettingValue::String("DuckDuckGo".to_string()),
    );

    // Appearance settings (additional to existing UI settings)
    defaults.insert(
        "appearance.toolbar_customization".to_string(),
        SettingValue::String("default".to_string()), // default, compact, minimal
    );

    // Privacy settings (additional)
    defaults.insert(
        "privacy.cookie_policy".to_string(),
        SettingValue::String("allow_all".to_string()), // allow_all, block_third_party, block_all
    );
    defaults.insert(
        "privacy.tracking_protection".to_string(),
        SettingValue::Boolean(true),
    );

    // Security settings (additional)
    defaults.insert(
        "security.safe_browsing".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "security.password_manager_enabled".to_string(),
        SettingValue::Boolean(true),
    );

    // Downloads settings
    defaults.insert(
        "downloads.default_location".to_string(),
        SettingValue::String("~/Downloads".to_string()),
    );
    defaults.insert(
        "downloads.ask_where_to_save".to_string(),
        SettingValue::Boolean(false),
    );

    // Advanced settings
    defaults.insert(
        "advanced.hardware_acceleration".to_string(),
        SettingValue::Boolean(true),
    );
    defaults.insert(
        "advanced.proxy_enabled".to_string(),
        SettingValue::Boolean(false),
    );
    defaults.insert(
        "advanced.proxy_host".to_string(),
        SettingValue::String("".to_string()),
    );
    defaults.insert(
        "advanced.proxy_port".to_string(),
        SettingValue::Integer(8080),
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

    #[test]
    fn test_defaults_network_quic_http3_settings() {
        let defaults = create_defaults();
        // QUIC setting (FEAT-100)
        assert_eq!(
            defaults.get("network.enable_quic"),
            Some(&SettingValue::Boolean(true))
        );
        // HTTP/3 setting (FEAT-100)
        assert_eq!(
            defaults.get("network.enable_http3"),
            Some(&SettingValue::Boolean(true))
        );
    }
}
