use crate::harness::TestContext;
use serial_test::serial;
use ssv::{generate, list, remove, show};

#[test]
#[serial]
fn managed_host_lifecycle_uses_public_api() {
    let context = TestContext::new();

    let generated = generate("flow.test", "ed25519", None, None).expect("generate should succeed");
    assert_eq!(show("flow.test").expect("show should succeed"), generated);
    assert_eq!(list().expect("list should succeed"), vec!["flow.test"]);

    remove("flow.test").expect("remove should succeed");
    assert!(!context.host_config("flow.test").exists());
    assert!(!context.private_key("ed25519", "flow.test").exists());
}
