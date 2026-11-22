//! The core lists the ids of the rules this crate hosts so that config
//! validation can recognise them. This keeps that list in step with the registry.

use lynter_core::config::CODE_RULE_IDS;

#[test]
fn core_lists_exactly_the_registered_code_rules() {
    let registered: Vec<String> = lynter_code::registry()
        .iter()
        .map(|r| r.id().to_owned())
        .collect();
    let listed: Vec<String> = CODE_RULE_IDS.iter().map(|id| (*id).to_owned()).collect();
    assert_eq!(listed, registered);
}
