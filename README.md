
## Create module
```bash
cargo new src/modules/{module_name} --lib
```

## Add to workspace in Cargo.toml
```toml
[workspace]
members = [
  ...
  "src/modules/{module_name}",
]
```

## Make testbed
```bash
cd src/modules/{module_name}/src
mkdir bin
touch bin/test.rs
```

## Run test
```bash
cargo run -p {module_name} --bin test
```

