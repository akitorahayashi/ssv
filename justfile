set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Generate code coverage report
coverage:
    rm -rf target/tarpaulin coverage
    mise exec -- cargo tarpaulin \
        --engine llvm \
        --target-dir target/tarpaulin \
        --out Stdout \
        --out Html \
        --output-dir coverage \
        --all-features \
        --fail-under 40
