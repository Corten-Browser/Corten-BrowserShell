// Integration test file for unit tests
// This allows cargo test to find and run the tests

mod unit {
    include!("unit/test_tab_manager.rs");

    // Include navigation tests in a separate module to avoid name conflicts
    mod navigation {
        include!("unit/test_navigation.rs");
    }

    // Include scalability tests
    mod scalability {
        include!("unit/test_scalability.rs");
    }
}
