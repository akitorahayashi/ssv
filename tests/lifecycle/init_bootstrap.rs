use crate::harness::TestContext;
use ssv::BootstrapStatus;
use std::fs;

#[test]
fn init_reports_created_then_up_to_date_via_library_api() {
    let context = TestContext::new();
    let ctx = context.ctx();

    assert_eq!(ctx.init().expect("init should succeed"), BootstrapStatus::Created);
    assert_eq!(ctx.init().expect("init should succeed again"), BootstrapStatus::UpToDate);

    let config = fs::read_to_string(context.main_config()).expect("config should exist");
    assert!(config.contains("Include ~/.ssh/conf.d/*.conf"));
}

#[test]
fn generate_also_bootstraps_main_config() {
    let context = TestContext::new();
    let ctx = context.ctx();

    ctx.generate("bootstrap.test", None, "ed25519", None, None).expect("generate should succeed");

    let config = fs::read_to_string(context.main_config()).expect("config should exist");
    assert!(config.contains("Include ~/.ssh/conf.d/*.conf"));
}
