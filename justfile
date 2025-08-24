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
