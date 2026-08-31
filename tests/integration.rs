mod harness;

mod cli {
    mod audit;
    mod authorize;
    mod generate;
    mod help_and_version;
    mod init;
    mod link;
    mod list;
    mod remove;
    mod set;
    mod show;
}

mod lifecycle {
    mod generate_list_remove;
    mod init_bootstrap;
    mod read_only_operations;
    mod removal_safety;
    mod show_failures;
}

mod audit {
    mod finding_collection;
    mod healthy_environment;
    mod identity_mismatch;
    mod missing_assets;
    mod orphaned_assets;
    mod permissions;
}

mod security {
    mod managed_host_contract;
    mod managed_path_boundary;
    mod symlinks;
}
