@_default:
  just -l

# Run all the tests
test:
  cargo test --doc
  cargo nextest run
