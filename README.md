# Um9i.dev

Source for <https://um9i.dev/> — a showcase of the things I'm working on.

Built with [axum](https://github.com/tokio-rs/axum) and
[askama](https://github.com/askama-rs/askama). Deployed as a container on
DigitalOcean App Platform.

## Run locally

```sh
cargo run                 # http://localhost:8080
PORT=3000 cargo run       # different port
```

## Adding a project

Edit [`content/site.toml`](content/site.toml) and add a `[[project]]` block.
Content is compiled into the binary, so `cargo test` catches mistakes (bad
slugs, duplicates, unknown status) before deploy.

## Layout

| Path            | What                                                      |
| --------------- | --------------------------------------------------------- |
| `src/main.rs`   | Startup, tracing, graceful shutdown on SIGTERM            |
| `src/routes.rs` | Routes, page templates, middleware (compression, CSP, …) |
| `src/content.rs`| Loads and validates `content/site.toml`, renders Markdown |
| `src/config.rs` | `HOST`, `PORT`, `STATIC_DIR` from the environment         |
| `templates/`    | askama templates, type-checked at compile time            |
| `static/`       | CSS and favicon, served at `/static`                      |

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test
```

## Deploy

```sh
docker build -t um9i . && docker run --rm -p 8080:8080 um9i   # test the image
doctl apps create --spec .do/app.yaml                          # first time
```

The spec in [`.do/app.yaml`](.do/app.yaml) mirrors the live app. With
`deploy_on_push`, every push to `master` redeploys. App Platform terminates TLS
and issues the certificate for `um9i.dev` once DNS points at the app.
