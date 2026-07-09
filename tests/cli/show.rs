use crate::harness::TestContext;

#[test]
fn show_outputs_only_public_key() {
    let context = TestContext::new();
    context.cli().args(["generate", "show.test"]).assert().success();

    context
        .cli()
        .args(["show", "show.test"])
        .assert()
        .success()
        .stdout("ssh-ed25519 AAAATESTKEY ed25519@ssv\n")
        .stderr("");
}
