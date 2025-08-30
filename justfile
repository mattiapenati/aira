@_default:
  just -l

# Generate documentation
doc *ARGS:
  RUSTDOCFLAGS="--cfg docsrs" \
  cargo +nightly doc --workspace --no-deps --features f16,f128 {{ARGS}}

# Create coverage report
coverage:
  cargo tarpaulin --workspace --out xml

# Run the whole benchmark suite
bench *ARGS:
  cargo bench -p aira-bench --bench aira-bench -- --output-format bencher {{ARGS}}

# Check local packages
check: check-byteorder check-tiff

# Run all the tests
test: test-byteorder test-tiff

# Run all checks of byteorder crate
[group('check')]
check-byteorder:
  cargo clippy -p aira-byteorder -- -D warnings
  cargo clippy -p aira-byteorder --no-default-features -- -D warnings
  cargo +nightly clippy -p aira-byteorder --features f16,f128 -- -D warnings

# Run all test of byteorder crate
[group('test')]
test-byteorder:
  cargo nextest run -p aira-byteorder
  cargo +nightly nextest run -p aira-byteorder --features f16,f128

# Run all checks of tiff crate
[group('check')]
check-tiff:
  cargo hack -p aira-tiff --feature-powerset --mutually-exclusive-features chrono,jiff clippy -- -D warnings

# Run all test of tiff crate
[group('test')]
test-tiff:
  cargo test -p aira-tiff --doc
  cargo nextest run -p aira-tiff
