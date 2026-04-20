# Crate predicates

Crate predicates specify which crates and versions a plugin, skill group, or individual skill applies to.

## Predicate syntax

A crate predicate is a crate name with an optional version requirement.

Examples:

- `serde`
- `serde>=1.0`
- `tokio^1.40`
- `regex<2.0`
- `serde=1.0`
- `serde==1.0.219`

Semantics:

- bare crate name: any version
- `>=`, `<=`, `>`, `<`, `^`, `~`: standard semver operators
- `=1.0`: compatible-version matching, equivalent to `^1.0`
- `==1.0.219`: exact-version matching

## Usage in different contexts

### Plugin manifests (TOML)

The `crates` field accepts a string or array:

- `crates = "serde"`
- `crates = ["serde", "tokio>=1.40"]` (any version of serde *or* versions of `tokio` `>=1.40`)
- `crates = ["*"]` (wildcard for all crates)

### Skill frontmatter (YAML)

The `crates` field uses comma-separated values:

- `crates: serde`: matches any version of serde
- `crates: serde, tokio>=1.40`: matches any version of serde *or* dependencies on tokio>=1.40

## Matching behavior

A `crates` predicate matches if *at least one* of the crates in its list matches against the workspace.

If there are multiple `crates` predicates in scope, all of them must match. For example with skills, `crates` predicates can appear at three distinct levels:

* If a [plugin](./plugin-definition.md) defines a `crates` predicate at the top-level, it must match before any other plugin contents will be considered.
* If a skill-group within a plugin defines a `crates` predicate, that predicate must match before the skills themselves will be fetched.
* If the skills define `crates` in their front-matter, those crates must match before the skills will be added to the project.
