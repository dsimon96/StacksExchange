# Development Setup

Download and install [PostgreSQL](https://www.postgresql.org/download/). Record
the password in a `.env` file in this directory:

```
DEF_db.password=hunter2
```

Install the Diesel CLI using cargo. You probably only want the 'postgres'
feature.

```
cargo install diesel_cli --no-default-features --features postgres
```

Add a `DATABASE_URL` to the `.env` file. Diesel CLI will use this to connect to
the database. (For more info on this, refer to the
[PostgreSQL docs](https://www.postgresql.org/download/)).

```
DATABASE_URL=postgres://postgres:hunter2@localhost/stacks_exchange
```

Now you can use Diesel CLI to set up the database with existing migrations:

```
diesel setup
```

# Running the server binary

`cargo run`'s default invocation will start a server listening on localhost,
port 8080.

```
cargo run
```

By default, graphiql is not exposed, to ensure that it is not accidentally
exposed in production. You can expose graphiql by enabling the 'graphiql'
feature:

```
cargo run --features graphiql
```

Then open `http://localhost:8080/graphiql` in your browser. If you prefer
GraphQL Playground, that's available at `http://localhost:8080/playground`.

## Autoreload

Autoreload can speed up the development cycle by automatically rebuilding and
re-running the server on any source or config changes. To enable autoreload,
you'll need to install cargo-watch and systemfd:

```
cargo install cargo-watch systemfd
```

To support autoreloading, the 'autoreload' feature must be enabled. Then you can
use systemfd to automatically invoke `cargo run` on any filesystem changes, and
forward http traffic from port 8080:

```
RUST_LOG=info systemfd --no-pid -s http::8080 -- cargo watch -x \
"run --features autoreload --features graphiql"
```

This command is rather complicated, but if you use VS Code as your editor (I
recommend the extensions Better TOML, crates, and rust-analyzer for rust
development), the task 'run development server' makes it easy.

# Configuring the server binary

The schema of configuration is defined in `settings::Settings` and its dependent
Structs. Configuration parameters which are declared there must be defined and
assigned a default value in `conf/default.toml`, so that configuration may be
safely deserialized. To create a different configuration 'flavor', create
another `.toml` file in the `conf/` folder containing only the overridden
values, and pass it to the binary via the `-c` flag. (This approach allows us to
safely add new configuration parameters to default.toml without requiring them
to be added to all 'flavors' immediately.) For example:

```
cargo run -- -c conf/prod.toml
```

You can additionally override individual values at the time of command
invocation by adding an environment variable of the form `DEF_param_name=val`.
These are applied after any overrides provided in the provided conf file. Values
in a named section of the config should be prefixed with the name of the
section, followed by a period. For convenience, you can define environment
variables in the `.env` file, which will be loaded at the beginning of the
program. This is mainly for the convenience of not having to provide the
environment variables repeatedly - refrain from access environment variables
directly without declaring them in `settings::Settings`.

```
DEF_foo=bar DEF_baz=fooey cargo run -- -c conf/prod.toml
```

The one exception is logging level, which is set via the `RUST_LOG` environment
variable. See top-level documentation for the env_logger crate for details. For
example, to view `INFO` logs from the actix_web logger:

```
RUST_LOG="actix_web=info" cargo run --features graphiql
```
