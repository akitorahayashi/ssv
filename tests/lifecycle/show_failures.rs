use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AppError, generate, show};
use std::fs;

#[test]
#[serial]
fn show_reports_missing_host() {
    let _context = TestContext::new();

    assert!(matches!(show("missing.test"), Err(AppError::HostNotFound(_))));
}

#[test]
#[serial]
fn show_rejects_ambiguous_identity_and_missing_public_key() {
    let context = TestContext::new();
    generate("ambiguous.test", "ed25519", None, None).expect("generate should succeed");
    let config = context.host_config("ambiguous.test");
    let mut contents = fs::read_to_string(&config).expect("config should be readable");
    contents.push_str("IdentityFile ~/.ssh/id_ed25519_other.test\n");
    fs::write(&config, contents).expect("config should be replaced");
    assert!(show("ambiguous.test").is_err());

    generate("missing-public.test", "ed25519", None, None).expect("generate should succeed");
    fs::remove_file(context.public_key("ed25519", "missing-public.test"))
        .expect("public key should be removed");
    assert!(show("missing-public.test").is_err());
}
