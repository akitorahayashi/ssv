use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn generate_outputs_public_key_and_creates_assets() {
    let context = TestContext::new();

    context
        .cli()
        .args(["generate", "--host", "github.com", "--user", "git", "--port", "2222"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated SSH assets for 'github.com'"))
        .stdout(predicate::str::contains("ssh-ed25519 AAAATESTKEY"));

    assert!(context.host_config("github.com").exists());
    assert!(context.private_key("ed25519", "github.com").exists());
}
