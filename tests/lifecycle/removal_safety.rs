use crate::harness::TestContext;
use serial_test::serial;
use ssv::{generate, remove};

#[test]
#[serial]
fn remove_missing_host_does_not_guess_key_names() {
    let context = TestContext::new();
    generate("foo.bar.com", "ed25519", None, None).expect("generate should succeed");

    assert!(remove("bar.com").is_err());
    assert!(context.private_key("ed25519", "foo.bar.com").exists());
}
