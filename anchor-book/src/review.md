# Reviewing this draft

Pick an anchor and read its property. Would changing that choice require a design conversation? Then read its criteria as examples. Each describes a concrete observation against a temporary fixture. To run one anchor from the repository root:

```sh
anchorbook verify --config anchorbook.toml --method test --anchor A-plugin-chains
```

To try the cycle by itself:

```sh
cargo nextest run -p symposium -E 'test(=sync_crate_metadata_cycle_detection)'
```

The fixture exercises `init` and `sync` and inspects the installed skill. The hook and agent examples likewise check Symposium’s replies or the files it writes. Launching a vendor agent is a separate experiment. A passing example applies to the case named in the criterion.

## Try a design choice at the CLI

What should `init` do if it cannot tell which agents you already configured? Try the [configuration anchor](design/configuration.md#A-config-unreadable) with a temporary user directory. From the repository root on Linux or macOS:

```sh
cargo build --bin symposium
trial_dir=$(mktemp -d)
printf '[[agent]\n' > "$trial_dir/config.toml"
SYMPOSIUM_HOME="$trial_dir" target/debug/symposium init --add-agent claude
cat "$trial_dir/config.toml"
```

`init` should exit with an error naming the file. The last command should print exactly the malformed line you supplied. You can edit the input and retry the same command to explore what counts as unreadable.

## Build and preview

The draft uses the published Anchor Book 0.1.0 crate and the existing mdBook tool versions. From the repository root:

```sh
cargo install anchor-book --version '=0.1.0' --locked --force
cargo install mdbook --version 0.5.4 --locked
cargo install mdbook-mermaid --version 0.17.1 --locked
cargo install cargo-nextest --version 0.9.151 --locked
anchorbook check --config anchorbook.toml
mdbook serve anchor-book --hostname 127.0.0.1 --port 3002
```

## Test coverage

The [unlinked test review](review/unlinked-tests.md) lists the existing anchors each test could support and the behavior its assertions leave to specify. The [original audit](review/test-audit.md) records coverage before the 15 additional anchors.

## Open design questions

- Can a loaded plugin register a new manager? Today manager setup precedes plugin resolution. A crate-defined root manager or recursive manager discovery would require a separate decision.
- Which predicate contexts should share custom definitions and cached results? The predicate chapter examples establish cross-plugin custom skill gates. Uniform behavior across commands and hooks remains open.
- What should happen if replacing a staged skill fails after removing the previous directory? The installation anchor covers failure while copying into staging. Continuous availability throughout replacement would require a stronger guarantee.
