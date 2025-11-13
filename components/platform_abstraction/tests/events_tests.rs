// @validates: REQ-003
//! Integration tests for platform event translation

use platform_abstraction::EventTranslator;

#[test]
#[should_panic(expected = "not yet implemented")]
fn test_event_translator_translate_fails_unimplemented() {
    // RED: Event translation not implemented

    // Given an event translator
    let translator = EventTranslator::new();

    // When we try to translate an event
    // Then it should panic (unimplemented)
    let _ = translator.translate("resize");
}
