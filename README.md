# KCL

Parser, generator, schema admission, and canonical digests for the Kense
Configuration Language, implemented as a single Rust crate.

> **Rust only.** This repository provides the `kense-kcl` crate (library name
> `kcl`). No SDKs, bindings, or documentation for other languages are provided.

## Overview

A lightweight, typed configuration language for agent configurations. KCL is
designed for defining structured agent behaviors with blocks, attributes, and
function calls.

## Features

- **KCL Block Syntax**: Labeled blocks, attributes, nested blocks, and function calls defined by the KCL grammar
- **Typed Values**: Strings, numbers, booleans, lists, objects, null, and function calls
- **Function Calls**: Built-in support for `env()`, `concat()`, and custom functions
- **Bidirectional**: Parse KCL text to AST, generate AST back to text
- **Type-Stable Strings**: Canonical generation quotes empty, numeric-looking,
  keyword-like, and Unicode strings so parsing cannot change their value kind
- **Stable Canonical Digests**: Canonical UTF-8 plus lowercase,
  algorithm-prefixed SHA-256 digests
- **Structured Diagnostics**: Stable wire codes, complete source spans,
  UTF-8 byte offsets, and messages that never echo source token values
- **Bounded Multi-Diagnostics**: Deterministic line recovery, configurable
  diagnostic budgets, and explicit truncation without changing fail-fast parsing
- **Schema Admission**: Closed-by-default document shapes, recursive value
  rules, stable logical paths, and bounded validation reports

## Syntax

```kcl
providers "openai" {
    api_key = env("OPENAI_API_KEY")
    base_url = "https://api.openai.com/v1"
}

default_model = "gpt-4"

settings {
    temperature = 0.7
    max_tokens = 2000
}

nested "label" {
    deeply {
        value = "supported"
    }
}
```

## Installation

> **Not yet published to crates.io.** Until the first release, depend on this
> repository directly. Once published, replace the dependency with
> `kense-kcl = "0.3.0"`.

Add the crate to your `Cargo.toml`. The package is named `kense-kcl`; the
library target (and therefore the name used in `use` statements) is `kcl`.

```toml
[dependencies]
kense-kcl = { git = "https://github.com/kense-labs/kcl", branch = "main" }
```

```rust
use kcl::{generate, parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = parse(r#"
        name = "test"
        count = 42
    "#)?;

    let output = generate(&document);
    println!("{output}");
    Ok(())
}
```

## API

### Parse

```rust
use kcl::{parse, Document, ParseError};

// parse(input: &str) -> Result<Document, ParseError>
let document = parse("name = \"test\"")?;
```

The high-level parsers apply resource limits before recursively parsing
untrusted input:

| Limit | Default |
| --- | ---: |
| UTF-8 document size | 1 MiB |
| Structural nesting depth | 64 |
| Items in one document or collection | 10,000 |
| UTF-8 source token size | 256 KiB |
| Collected diagnostics | 100 |

Use explicit limits at API admission boundaries:

```rust
use kcl::{parse_with_limits, ParseLimits};

let document = parse_with_limits(
    input,
    ParseLimits {
        max_document_bytes: 64 * 1024,
        max_nesting_depth: 32,
        max_collection_items: 1_000,
        max_token_bytes: 16 * 1024,
        max_diagnostics: 20,
    },
)?;
```

Document and token sizes count UTF-8 bytes. Nesting includes blocks, lists,
objects, and function calls. Collection limits apply independently to the
document, each block body and label list, each list or object, and each
function argument list. Direct lexer use is an advanced API and is not a
substitute for the bounded high-level parser.

### Parse diagnostics

Diagnostics use stable codes and complete source spans:

| Code | Meaning |
| --- | --- |
| `kcl.limit.document_bytes` | Document byte limit exceeded |
| `kcl.limit.token_bytes` | Source token byte limit exceeded |
| `kcl.limit.nesting_depth` | Structural nesting limit exceeded |
| `kcl.limit.collection_items` | Collection item limit exceeded |
| `kcl.parse.unexpected_token` | Token is not valid at this grammar position |
| `kcl.parse.expected_token` | A required delimiter or token kind is missing |
| `kcl.parse.unexpected_eof` | Input ended before the current construct |

Locations use one-based lines and columns. Span offsets are zero-based UTF-8
byte offsets, so offsets stay stable even when Unicode precedes an error.
The compatibility `line` and `column` fields equal `span.start.line` and
`span.start.column`.

```rust
use kcl::{parse, DiagnosticCode};

let error = parse(r#""private-value""#).unwrap_err();
assert_eq!(error.code, DiagnosticCode::UnexpectedToken);
assert_eq!(error.code.as_str(), "kcl.parse.unexpected_token");
assert_eq!(error.span.start.offset, 0);
```

Diagnostics identify token kinds but never include token values or source
snippets. Callers should preserve that boundary and must not attach the
untrusted document to API errors or logs.

The `parse` APIs remain fail-fast. CLI and editor integrations can collect
multiple errors without constructing a partial AST:

```rust
use kcl::{collect_diagnostics_with_limits, ParseLimits};

let report = collect_diagnostics_with_limits(
    "first = ]\nsecond = ]",
    ParseLimits {
        max_diagnostics: 20,
        ..ParseLimits::default()
    },
);
```

After a syntax error, collection resumes at the next source line. Resource-limit
diagnostics remain fatal and appear at most once. The collector stores no more
than `max_diagnostics` errors and sets `truncated` only after observing an
additional error beyond that budget. A zero budget therefore returns no
diagnostics and sets `truncated` when the input is invalid.

### Schema admission

Parse untrusted input with explicit limits, then validate the resulting
document before activation:

```rust
use kcl::{
    parse_with_limits, validate_document_with_limits, AttributeSchema,
    ParseLimits, Schema, ValueSchema,
};

let schema = Schema::new().attribute(
    "version",
    AttributeSchema::required(ValueSchema::number()),
);
let limits = ParseLimits {
    max_diagnostics: 20,
    ..ParseLimits::default()
};
let document = parse_with_limits("version = 1", limits)?;
let report = validate_document_with_limits(&document, &schema, limits);
assert!(report.is_empty());
```

Schemas are closed by default. They can declare required or optional
attributes, nested block and label cardinalities, whether matching block
occurrences are semantically unordered, and recursive `Any`, `String`,
`Number`, `Bool`, `Null`, `List`, `Object`, `Call`, and `OneOf` value rules.
Unknown attributes, blocks, and object fields require an explicit allow flag.

Schema diagnostics use stable `kcl.schema.*` codes and logical paths such as
`$.blocks.provider[0].attributes.api_key`. Messages and paths never include
attribute values, call arguments, or block labels. Reports use the same
diagnostic budget as parsing and set `truncated` only after observing an
additional schema error beyond that budget.

Schema validation checks document shape, not host-specific semantics such as
numeric ranges, secret resolution, or provider credentials. Those checks
remain the responsibility of the admitting component.

### Generate

```rust
use kcl::generate;

// generate(doc: &Document) -> String
let output = generate(&document);
```

The generator emits native KCL syntax. Block labels remain in block headers;
there is no label-as-attribute compatibility output.

### Canonical bytes and digest

Use the canonical APIs when KCL bytes are signed, stored, or compared:

```rust
use kcl::{canonical_bytes, canonical_digest, parse};

let document = parse("limits { memory = 128000000 }")?;
let bytes = canonical_bytes(&document)?;
let digest = canonical_digest(&document)?;
assert!(digest.starts_with("sha256:"));
```

Canonical bytes use the default KCL generator, UTF-8 without a byte-order
mark, LF line endings, and exactly one final LF. Attribute maps and object
pairs are semantically unordered: their portable ASCII identifiers are sorted
by ascending byte value, and duplicate object keys use the last value.
Document-level attributes remain assignments for every value kind, while an
unlabeled block with one differently named attribute retains its block braces;
canonicalization never changes one shape into the other.
Document and nested-block order, block-label order, list items, and function
arguments remain ordered and therefore affect the digest.

After schema admission, use the schema-aware APIs to normalize repeatable block
types that the trusted schema marks as unordered:

```rust
use kcl::{canonical_digest_with_schema, BlockSchema, Schema};

let schema = Schema::new().block(
    "provider",
    BlockSchema::new(Schema::new()).unordered(true),
);
let digest = canonical_digest_with_schema(&document, &schema)?;
```

Normalization is recursive. For each body, only occurrences of the same
declared block name whose rule sets `unordered` are sorted by canonical UTF-8
bytes. Their existing positions are retained, so other block types and unknown
blocks remain ordered. Schema-aware canonicalization does not perform
admission; validate the document first.

Finite numbers use the shortest round-tripping decimal representation,
including `0` for negative zero and stable exponent boundaries. Comments and
source whitespace are discarded by parsing, while string and label Unicode
scalar sequences are preserved without NFC/NFD normalization. Programmatic
non-finite numbers and non-portable identifiers fail with redacted
`CanonicalError` values. Digests are lowercase
`sha256:<64 hexadecimal characters>` strings over the exact canonical bytes.
The cases under `fixtures/canonical/digest-cases.json` and
`fixtures/canonical/schema-block-order-cases.json` are the compatibility
fixtures used by the test suite.

### Value constructors

All constructors live in `kcl::builder`:

| Kind | Rust |
|------|-------------|
| String | `string("x")` |
| Number | `number(42.0)` / `integer(42)` |
| Boolean | `boolean(true)` |
| Null | `null()` |
| List | `list(vec![...])` |
| Function call | `call("env", vec![...])` |

### Builders

```rust
use kcl::builder::{number, string, BlockBuilder, DocumentBuilder};

let nested_block = BlockBuilder::new("nested").build();

let block = BlockBuilder::new("config")
    .label("primary")
    .attr("name", string("test"))
    .attr("count", number(42.0))
    .nested_block(nested_block)
    .build();

let document = DocumentBuilder::new()
    .block(block)
    .build();
```

## Value Types

| Kind | Description |
|------|-------------|
| `String` | Quoted text: `"hello"` |
| `Number` | Integer or float: `42`, `3.14` |
| `Bool` | Boolean: `true`, `false` |
| `Null` | Null value: `null` |
| `List` | Ordered collection: `[1, 2, 3]` |
| `Object` | Key-value pairs: `{key = value}` |
| `Call` | Function invocation: `env("VAR")` |

Canonical generation always quotes `String` values. For example, `""`, `"42"`,
`"1.88"`, `"true"`, and `"null"` remain strings after a parse/generate/parse
round trip. The test suite validates this rule against the fixture under
`fixtures/canonical/string-values.kcl`.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo doc --no-deps
```

The CI workflow runs all four checks plus `cargo package --allow-dirty` on the
`main` branch. The minimum supported Rust version is 1.70.

## License

MIT
