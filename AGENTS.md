# AGENTS.md — KCL (`kense-labs/kcl`)

## Repository identity

- **This is a standalone project.** KCL is its own Git repository and its own
  release unit; it is not a directory, crate or submodule of `kbox` or `krun`,
  and it is not a sibling `../kcl` working tree on a developer machine.
- Single Rust library crate: package `kense-kcl`, library target `kcl`,
  edition 2021, MSRV 1.70, MIT. `Cargo.lock` is intentionally not committed
  (`.gitignore`); this is a library, not an application workspace.
- **Rust only.** No SDKs, bindings or documentation for other languages.
  `parse_acl` / `generate_acl` are `#[doc(hidden)]` compatibility aliases for
  older call sites; they do not make this an "ACL" product and must not be
  reintroduced into public docs or examples.
- Runtime dependencies are deliberately minimal: `ryu-js 1.0.3` and
  `sha2 0.10`. `serde` / `serde_json` are dev-only, used to read fixtures.

## Downstream consumers and compatibility contract

- `kbox` and `krun` both depend on this repository at a **pinned `git rev`**
  (`76bf580454f1d69d7b826223ac50161efa685f85` as of 2026-11-12). They must pin
  the same revision; a local `path = "../kcl"` override is forbidden. `kbox`
  enforces this with `scripts/check-kcl-source.py` (`--krun` for the
  cross-repository check).
- Changing the pinned rev is a coordinated change across **both** manifests,
  **both** `Cargo.lock` files, the gate, and the active specs in both repos.
  Never advance only one consumer.
- Public surface actually used downstream (verified 2026-11-12):
  - `krun`: `kcl::parse`, `Block`, `Value`.
  - `kbox`: `parse_with_limits`, `validate_document`,
    `canonical_bytes_with_schema`, `canonical_digest_with_schema`,
    `Schema`, `SchemaDiagnosticCode`, `AttributeSchema`, `BlockSchema`,
    `Cardinality`, `ValueSchema`, `ParseLimits`, `CanonicalError`,
    `Document`, `Block`, `Value`.
  Removing or renaming any of these is a breaking change requiring a
  coordinated downstream edit.
- **Canonical bytes / digests are wire contracts.** `kbox` uses
  `canonical_digest_with_schema` as the immutable identity of an OCI build
  plan. Any change to generator output, key sorting, number rendering, block
  ordering, schema normalization or digest prefix silently changes plan
  identities and cache keys. Treat such changes as breaking: update
  `fixtures/canonical/*`, downstream expectations, and `CHANGELOG.md`
  together.
- Diagnostic code strings (`kcl.limit.*`, `kcl.parse.*`, `kcl.schema.*`) are
  stable wire values. Do not rename them back to `acl.*`. Diagnostics must
  never echo source token values, attribute values, call arguments or block
  labels; spans keep zero-based UTF-8 byte offsets and one-based line/column.

## Layout

- `src/` — parser, lexer, AST, generator, canonicalization, schema admission,
  diagnostics and parse limits (single crate, flat module tree).
- `tests/`, `fixtures/`, `examples/` — integration tests, shared JSON/KCL
  fixtures, and the `parse_generate` example. Fixtures are the compatibility
  oracle; edit them only alongside the behaviour they encode.
- `.github/workflows/ci.yml`, `CHANGELOG.md`, `LICENSE`, `README.md`.
- Do not add a second crate, a nested workspace, a `Cargo.lock`, or build
  scripts / code generation steps.

## Code rules

- Keep the crate dependency-free at runtime beyond the two declared crates;
  justify any new dependency in `CHANGELOG.md`.
- Respect MSRV 1.70 (no newer std APIs without raising `rust-version`
  deliberately).
- Parsing untrusted input must stay bounded: `parse_with_limits` /
  `collect_diagnostics_with_limits` with explicit `ParseLimits` at admission
  boundaries; resource-limit diagnostics are fatal and fail fast.
- Schemas stay closed by default. Validation checks document shape only;
  host semantics (numeric ranges, secrets, credentials) belong to the
  admitting component, not to this crate.
- Record user-visible changes in `CHANGELOG.md` (Keep a Changelog, SemVer).

## Verification (same steps as CI)

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo doc --no-deps
cargo package --allow-dirty
```

CI also sets `RUSTFLAGS=-Dwarnings`. Run the full set before proposing a
revision bump to downstream consumers.

## Do not

- Do not describe this crate as an SDK, a Node.js/TypeScript package, or part
  of kbox / krun.
- Do not reintroduce ACL naming, `acl.*` wire codes, or the removed
  `@kense-labs/acl` install instructions.
- Do not make downstream repositories point at a local checkout; they pin a
  git revision so both products always parse with one implementation.

## Local note (not part of the repository contract)

Some developer machines may also contain a sibling directory named `kcl`.
Such a directory is **not** this project: one known example is a non-Git,
13-file stand-in exported from kbox's historical `crates/acl` in November
2026 (its README is still titled `# kacl`). No manifest references it. Treat
this repository as the authoritative source and never link a local stand-in
as a `path = "../kcl"` dependency.
