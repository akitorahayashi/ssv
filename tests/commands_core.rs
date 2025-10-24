mod common;

use common::TestContext;
use serial_test::serial;
use ssv::generate;
use std::io;

#[test]
#[serial]
fn generate_with_invalid_host_surfaces_error() {
    let ctx = TestContext::new();

    ctx.with_dir(ctx.work_dir(), || {
        let err =
            generate("invalid/host", "ed25519", None, None).expect_err("invalid host should fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    });
}
