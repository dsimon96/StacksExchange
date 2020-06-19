# Running the server binary
`cargo run`'s default invocation will start a server listening on the ipv4
loopback address (127.0.0.1), port 8080.
```
cargo run
```
By default, graphiql is not exposed, to ensure that it is not accidentally
exposed in production. You can expose graphiql by enabling the 'graphiql'
feature:
```
cargo run --features graphiql
```

# Configuring the server binary

Configuration parameters must be defined and assigned a default value in
conf/default.toml, so that values may be accessed via `.unwrap()`. To create a
different configuration 'flavor', create another .toml file in the conf/
folder containing only the overridden values, and pass it to the binary via
the `-c` flag. (This approach allows us to safely add new configuration
parameters to default.toml without requiring them to be added to all 'flavors'
immediately.) For example:
```
cargo run -- -c conf/prod.toml
```
You can additionally override individual values at the time of command
invocation by adding an environment variable of the form `DEF_param_name=val`.
These are applied after any overrides provided in the provided conf file.
```
DEF_foo=bar cargo run -- -c conf/prod.toml
```

The one exception is logging level, which is set via the `RUST_LOG`
environment variable. See top-level documentation for the env_logger
crate for details. For example, to view `INFO` logs from the actix_web logger:
```
RUST_LOG="actix_web=info" cargo run --features graphiql
```
