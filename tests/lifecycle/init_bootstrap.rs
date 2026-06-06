use crate::harness::TestContext;
use serial_test::serial;
use ssv::{BootstrapStatus, generate, init};
use std::fs;

#[test]
#[serial]
fn init_reports_created_then_up_to_date_via_library_api() {
    let context = TestContext::new();

    assert_eq!(init().expect("init should succeed"), BootstrapStatus::Created);
    assert_eq!(init().expect("init should succeed again"), BootstrapStatus::UpToDate);

    let config = fs::read_to_string(context.main_config()).expect("config should exist");
    assert!(config.contains("Include ~/.ssh/conf.d/*.conf"));
}

#[test]
#[serial]
fn generate_also_bootstraps_main_config() {
    let context = TestContext::new();

    generate("bootstrap.test", "ed25519", None, None).expect("generate should succeed");

    let config = fs::read_to_string(context.main_config()).expect("config should exist");
    assert!(config.contains("Include ~/.ssh/conf.d/*.conf"));
}
