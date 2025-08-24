@_default:
  just -l

# Build the project using all the features
build:
  cargo hack --feature-powerset --mutually-exclusive-features chrono,jiff build

# Check local packages
check:
  cargo hack --feature-powerset --mutually-exclusive-features chrono,jiff check --workspace --all-targets
  cargo hack --feature-powerset --mutually-exclusive-features chrono,jiff clippy -- -D warnings

# Run all the tests
test:
  cargo test --doc
  cargo nextest run

# Create coverage report
coverage:
  cargo tarpaulin --workspace --out xml

# Run the whole benchmark suite
bench *ARGS:
  cargo bench -p aira-bench --bench aira-bench -- --output-format bencher {{ARGS}}
