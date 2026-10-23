# Contributing to Zynx

This is an experimental Rust workspace. Focused fixes, protocol regression tests,
working examples, and documentation improvements are welcome. Please follow the
[Code of Conduct](./CODE_OF_CONDUCT.md).

## Set up the workspace

1. Install the current stable Rust toolchain with the `rustfmt` and `clippy`
   components.
2. Clone the repository and run `cargo test --workspace --all-features`.
3. To run a real bot, copy `.env.example` to `.env`, set `DISCORD_TOKEN`, and follow
   the [README](./README.md). Tests do not need a token.

The workspace contains:

- [`zynx`](./zynx): client, Gateway, HTTP, event routing, and models.
- [`zynx-macros`](./zynx-macros): the `#[event_handler]` attribute.

The runtime depends on the local macro crate, so workspace tests exercise macro
changes in the same checkout.

## Before submitting a pull request

Run the same checks as CI:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-targets
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc --all-features
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links -D rustdoc::invalid_html_tags -D rustdoc::bare_urls" cargo doc --workspace --no-deps --all-features
```

Use `cargo fmt --all` to apply formatting. Add a regression test for behavioral
fixes, especially protocol decoding, session recovery, routing, and rate limits.
Keep tests independent of Discord credentials and services. Use mock responses
and controlled clocks where practical.

Describe the problem, the resulting behavior, and the checks you ran. Note public
API changes and update the README or examples when usage changes. Keep commits
focused and use a short message describing the change. Avoid unrelated model or
formatting rewrites in behavioral patches.

The default and `examples` feature configurations are tested on stable Rust.
An older minimum supported Rust version is not currently guaranteed.

## API documentation

Use Rustdoc Markdown in `///` comments for public items and `//!` comments for
modules. Link Rust items with intra-doc links such as
`[APIUser](crate::models::payloads::APIUser)` and wrap standalone URLs in angle
brackets. Use Markdown paragraphs or headings for defaults, deprecation notes,
parameters, and return values instead of TypeScript/JSDoc tags. Mark code fences
with their language, and use `no_run` for Rust examples that require a bot token.

The documentation build checks links, HTML tags, and bare URLs. Open
`target/doc/zynx/index.html` to inspect the same Rustdoc format used by docs.rs.

## Report a bug or propose a feature

Open a [GitHub issue](https://github.com/andrewdotdev/zynx/issues) with:

- The commit or crate version, Rust version, and operating system.
- Expected and observed behavior.
- A minimal reproduction or sanitized protocol fixture.

Remove bot tokens and private user data from logs and payloads before posting.
For sensitive vulnerabilities, use GitHub's private vulnerability reporting if
available. If it is unavailable, ask for a private reporting channel without
publishing the vulnerability details.

For larger features, explain the use case and proposed public API in an issue
before investing in an implementation.

## Attribution

Retain existing license headers when changing the ported API models. Those files
retain the upstream MIT license; the runtime and macro crates use Apache-2.0.
See [NOTICE.md](./NOTICE.md) for attribution details.
