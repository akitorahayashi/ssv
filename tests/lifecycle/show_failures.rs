use crate::harness::TestContext;
use ssv::AppError;
use std::fs;

#[test]
fn show_reports_missing_host() {
    let context = TestContext::new();
    let ctx = context.ctx();

    assert!(matches!(ctx.show("missing.test"), Err(AppError::HostNotFound(_))));
}

#[test]
fn show_rejects_ambiguous_identity_and_missing_public_key() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("ambiguous.test", None, "ed25519", None, None).expect("generate should succeed");
    let config = context.host_config("ambiguous.test");
    let mut contents = fs::read_to_string(&config).expect("config should be readable");
    contents.push_str("IdentityFile ~/.ssh/id_ed25519_other.test\n");
    fs::write(&config, contents).expect("config should be replaced");
    assert!(ctx.show("ambiguous.test").is_err());

    ctx.generate("missing-public.test", None, "ed25519", None, None)
        .expect("generate should succeed");
    fs::remove_file(context.public_key("ed25519", "missing-public.test"))
        .expect("public key should be removed");
    assert!(ctx.show("missing-public.test").is_err());
}
