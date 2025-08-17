@_default:
  just -l

# Run all the tests
test:
  cargo test --doc
  cargo nextest run

# Build the project using all the features
build:
  cargo hack --feature-powerset --mutually-exclusive-features chrono,jiff build
