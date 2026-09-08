# AXIOM

**One application. One language. Multiple execution domains.**

AXIOM is a general-purpose, statically typed programming language exploring a different way to build complex applications.

Modern software is usually split across multiple languages and systems:

```text
Frontend       → TypeScript / JavaScript
Backend        → Rust / Go / Python / Node
Database       → SQL
Workers        → another runtime
Communication  → REST / GraphQL / RPC / WebSockets
```

The application is logically one system, but the developer has to maintain the boundaries between all of these pieces.

AXIOM explores the opposite approach:

```text
                    AXIOM APPLICATION

             ┌───────────┼───────────┐
             │           │           │
          CLIENT       SERVER      WORKER
             │           │           │
             └───────────┼───────────┘
                         │
                      DATABASE
```

The entire application is written in AXIOM.

The execution environments still exist. They are explicit.

The difference is that the **language and compiler understand those boundaries**.

---

## The Idea

AXIOM's central goal is to reduce the cognitive and engineering overhead of building one application across multiple execution environments.

Instead of manually maintaining:

* duplicated types
* API contracts
* DTOs
* serialization code
* RPC layers
* client/server boundaries
* separate language tooling
* multiple type systems

AXIOM aims to make these relationships part of the language and compiler.

The developer should be able to reason about the entire application as one statically typed program.

> **Abstract the machinery, not the architecture.**

AXIOM does not hide where code executes.

It makes execution boundaries explicit while letting the compiler handle the repetitive machinery around them.

---

## The Core Concept: Execution Domains

Code belongs to an execution domain.

For example:

```text
domain server {
    ...
}

domain client {
    ...
}
```

The compiler tracks where functions execute and verifies interactions between domains.

A call inside the same domain is a normal function call.

A call across domains is a different operation:

```text
client
   │
   │ CrossDomainCall
   ▼
server
```

This distinction exists in the compiler's intermediate representation rather than being treated as ordinary application code.

Currently AXIOM supports the `client` and `server` domains as an experimental vertical slice.

---

## Typed Boundaries

Consider:

```text
type User {
    id: Int
    name: String
}

domain server {
    fn get_user(id: Int) -> User {
        return User {
            id: id,
            name: "Sohail"
        }
    }
}

domain client {
    fn main() {
        let user = get_user(42)
        print(user.name)
    }
}
```

The developer writes one language and one `User` type.

The compiler understands:

```text
main       → client
get_user   → server
User       → transmissible
get_user() → cross-domain call
```

The current runtime simulates the boundary locally.

The long-term model is:

```text
CLIENT
   │
   │ typed request
   ▼
boundary
   │
   │ serialized value
   ▼
SERVER
   │
   │ execute get_user()
   ▼
boundary
   │
   │ typed response
   ▼
CLIENT
```

The goal is to remove the need for developers to manually build the glue between these pieces.

---

## Transmissibility

Not every value should be allowed to cross an execution boundary.

AXIOM therefore treats **transmissibility as a type-level property**.

Currently transmissible:

```text
Int
Float
Bool
String
Unit
Array[T]       if T is transmissible
maybe T        if T is transmissible
Struct         if all fields are transmissible
```

Conceptually:

```text
User
├── id: Int
└── name: String

        ↓

   transmissible
```

Domain-local state should not accidentally cross boundaries.

For example, a future server-local database connection should never silently become client data.

The compiler should reject that relationship rather than relying on runtime checks.

---

## Why AXIOM?

The problem is not that developers cannot learn multiple languages.

The problem is that **one application increasingly requires developers to maintain several independent programming models at once**.

A typical feature might require:

```text
Frontend
    ↓
TypeScript type
    ↓
HTTP request
    ↓
Backend DTO
    ↓
Backend type
    ↓
Serialization
    ↓
Database model
    ↓
SQL schema
```

The same conceptual data may be represented several times.

AXIOM's long-term goal is to collapse those unnecessary boundaries.

Instead of thinking:

```text
frontend + backend + API + database
```

the developer thinks:

```text
one application
```

while the compiler still understands:

```text
where each part runs
what it can access
what can cross a boundary
how domains communicate
```

---

# Compiler Architecture

AXIOM is being built as a real compiler rather than a transpiler or syntax layer.

Current pipeline:

```text
Source
   ↓
Lexer
   ↓
Parser / AST
   ↓
Name Resolution
   ↓
Type Checking
   ↓
Domain Analysis
   ↓
HIR
   ↓
Bytecode
   ↓
Virtual Machine
```

### Lexer

Tokenizes AXIOM source code.

### Parser / AST

Builds a structured representation of the program.

### Name Resolution

Resolves identifiers, scopes, functions, and types.

### Type Checking

Performs static type checking.

### Domain Analysis

Determines execution domains and validates domain boundaries.

### HIR

Provides a semantic intermediate representation.

Cross-domain calls are represented explicitly as:

```text
CrossDomainCall {
    func,
    args,
    from_domain,
    to_domain
}
```

### Bytecode

Compiles supported HIR into AXIOM VM bytecode.

### Virtual Machine

Executes the generated bytecode.

---

# Current State

AXIOM is an **experimental compiler/language project under active development**.

The current implementation includes:

* Lexer
* Parser
* AST
* Name resolution
* Static type checking
* Structs
* Enums
* Functions
* Arrays
* `maybe`
* Control flow
* HIR
* Bytecode generation
* Virtual machine
* Execution domains
* Domain analysis
* Cross-domain call representation
* Recursive transmissibility checking
* Compiler diagnostics for invalid domain interactions

The current domain system is deliberately limited.

Cross-domain calls are currently simulated locally by the VM. Actual networking, serialization transport, and deployment across separate runtimes are future work.

---

# Current Domain Model

Currently:

```text
client
server
```

Supported:

```text
client → server
```

with compiler-verified transmissibility.

Direct:

```text
server → client
```

calls are rejected.

The eventual model is expected to expand toward:

```text
client
server
worker
native
database
```

but those are not implemented yet.

---

# Language

Basic AXIOM syntax currently includes:

### Functions

```text
fn add(a: Int, b: Int) -> Int {
    return a + b;
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
    print("x is less than y");
} else {
    print("x is not less than y");
}

while i < 5 {
    print(i);
    i = i + 1;
}
```

### Structs

```text
type User {
    id: Int
    name: String
}
```

### Arrays

```text
List[Int]
```

### Optional Values

```text
maybe User
```

### Execution Domains

```text
domain server {
    fn get_user(id: Int) -> User {
        ...
    }
}

domain client {
    fn main() {
        ...
    }
}
```

---

# Roadmap

AXIOM is being developed around the application model rather than around feature count.

### Completed

* Compiler foundation
* Static type system foundation
* AST / HIR pipeline
* Bytecode VM
* Execution domains
* Domain analysis
* Cross-domain call representation
* Recursive transmissibility analysis

### Next

* Strict domain entry points
* Real boundary/message representation
* In-memory RPC simulation
* AXIOM value serialization
* Stronger domain-local type semantics
* Typed domain events/messages
* Improved compiler diagnostics

### Later

The larger direction includes:

```text
                    AXIOM

        ┌───────────┼───────────┐
        │           │           │
      CLIENT       SERVER      WORKER
        │           │           │
        └───────────┼───────────┘
                    │
                 DATABASE
```

with the same language and type system spanning the application.

A future database model may allow application data definitions to live directly in AXIOM source, for example:

```text
app.db.ax
```

rather than requiring a separate language and duplicated application models.

Native and WASM compilation are also planned directions.

These are architectural goals, not claims about the current implementation.

---

# Design Principles

### One Application

The application should be understandable as one program rather than a collection of disconnected language ecosystems.

### One Type System

A type should not need to be recreated simply because it crosses an application boundary.

### Explicit Boundaries

AXIOM does not hide execution environments. The compiler knows where code runs.

### Compiler-Verified Boundaries

Invalid domain interactions should fail during compilation rather than becoming runtime surprises.

### Safety by Default

Values and operations should not cross execution boundaries unless the type system permits them.

### No Unnecessary Magic

AXIOM should automate repetitive infrastructure while keeping application architecture visible to the developer.

### General Purpose

AXIOM is intended to remain a general-purpose language rather than becoming a framework-specific DSL.

---

# Building

### Prerequisites

* Rust toolchain
* MSVC Build Tools on Windows
* GCC/Clang on Linux/macOS

### Build

```bash
git clone https://github.com/sohailcodes-ai/axiom.git
cd axiom

cargo build --release
```

### Run

```bash
cargo run -- run examples/hello.ax
```

---

# Repository

```text
axiom/
├── compiler/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── tokens.rs
│       ├── lexer.rs
│       ├── ast.rs
│       ├── parser.rs
│       ├── name_resolver.rs
│       ├── type_checker.rs
│       ├── domain_analysis.rs
│       ├── hir.rs
│       ├── hir_lower.rs
│       ├── bytecode.rs
│       ├── codegen.rs
│       ├── vm.rs
│       └── errors.rs
├── docs/
├── examples/
└── reference/
```

---

# Project Status

AXIOM is experimental.

It is not a production-ready replacement for TypeScript, Rust, Go, SQL, or existing application stacks.

The purpose of the project is to explore whether **the compiler can understand an entire application's execution model instead of developers manually maintaining every boundary between its parts.**

That is the problem AXIOM is trying to solve.

---

## License

MIT
