# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Build and run on example directory
example: build
    ./target/debug/md-wiki example output

# Clean build artifacts and output
clean:
    cargo clean
    rm -rf output
