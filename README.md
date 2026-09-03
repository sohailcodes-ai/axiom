# AXIOM

A general-purpose, statically-typed programming language for the full stack.

## What is AXIOM?

AXIOM is a programming language designed for building software across the entire stack: CLI applications, backend services, web frontends, distributed systems, and WASM modules — all from a single language with a single type system.

## AXIOM's Differentiator

AXIOM's primary language-level differentiator is **compiler-verified typed execution boundaries**. The compiler statically verifies that code operates within the capabilities of its declared execution domain. Server-only code cannot accidentally end up in a client bundle. This is verified at compile time, not by a linter or runtime check.

## Design Principles

- **General Purpose** — Not optimized for one domain at the expense of others
- **Full Stack** — Client, server, worker, and native code in one language
- **One Type System** — Types are coherent across application boundaries
- **Explicit Execution Boundaries** — Code declares where it runs; the compiler verifies it
- **Safety by Default** — Ownership, null safety, and error handling are built in
- **Zero Mandatory Cost** — The core toolchain is free and open source
- **No Magic** — What happens is visible in the source code

## Current Status

AXIOM is in **Phase 1: Compiler Foundation**. The Rust compiler implementation has begun.

See [`docs/architecture.md`](docs/architecture.md) for the full technical design.
See [`docs/current-implementation-audit.md`](docs/current-implementation-audit.md) for the current implementation status.

## Building

### Prerequisites

- Rust toolchain (1.70+)
- MSVC Build Tools (Windows) or GCC (Linux/Mac)

### Build

```bash
# Clone the repository
git clone https://github.com/AxiomLang/Axiom.git
cd Axiom

# Build the compiler
cargo build --release

# Run a program
cargo run -- run examples/hello.ax
```

## Example

```text
fn main() {
    print("Hello, AXIOM!");
}
```

## Repository Structure

```
axiom/
├── compiler/                  # Compiler implementation (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # CLI entry point
│       ├── tokens.rs          # Token definitions
│       ├── lexer.rs           # Lexer
│       ├── ast.rs             # AST node definitions
│       ├── parser.rs          # Parser
│       ├── name_resolver.rs   # Name resolution
│       ├── type_checker.rs    # Type checking
│       ├── hir.rs             # HIR definitions
│       ├── hir_lower.rs       # HIR lowering
│       ├── bytecode.rs        # Bytecode definitions
│       ├── codegen.rs         # Bytecode generation
│       ├── vm.rs              # Virtual machine
│       └── errors.rs          # Error types
├── docs/                      # Design documents
├── examples/                  # Example programs
└── reference/                 # Python reference implementation
```

## Language Features

### Functions

```text
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

### Bindings

```text
let x = 42;
let mut y = 10;
```

### Control Flow

```text
if x < y {
    print("x is less than y")
} else {
    print("x is not less than y")
}

while i < 5 {
    print(i)
    i = i + 1
}
```

### Generics

```text
List[Int]
```

### Absence

```text
maybe User
```

### Execution Domains (Future)

```text
domain server {
    ...
}

domain client {
    ...
}
```

## License

MIT
