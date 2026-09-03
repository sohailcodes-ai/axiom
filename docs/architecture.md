# AXIOM Architecture - Phase 0 (Revised)

## Decision Classification

Every decision in this document is classified as:

- **LOCKED** - Final. Will not change without documented justification.
- **PROPOSED** - Best current direction. May change based on evidence.
- **OPEN** - Research question. Requires further investigation.

---

## A. Architectural Overview

AXIOM is a general-purpose, statically-typed programming language. A single AXIOM source program may contain code destined for multiple execution environments (server, client, worker, native). The compiler statically verifies that code operates within the capabilities of its declared execution domain. Values that cross domain boundaries are verified to be transmissible.

The compiler pipeline:

```
Source Code
  -> Lexer (tokens)
  -> Parser (AST)
  -> Name Resolution (resolved AST)
  -> Type Checking (typed AST)
  -> Domain Analysis (execution boundary verification)
  -> Lowering (HIR)
  -> Optimization (HIR)
  -> Bytecode Emission (VM bytecode)
  -> [Future: native code, WASM]
```

Each stage consumes and produces a distinct data structure. No stage reaches back into a prior representation.

---

## B. Execution Model

### Semantic Requirements (LOCKED)

1. An AXIOM program contains code that executes in different environments with different capabilities.
2. The compiler must know which environment each function executes in.
3. The compiler must verify that a function only uses operations available in its declared environment.
4. When a value crosses from one environment to another, the compiler must verify the value is transmissible.
5. The programmer must be able to see, from the source code, which environment a function executes in.
6. Cross-environment calls are not direct function calls - they are message exchanges.

### Proposed Source Representation (PROPOSED)

The exact syntax for declaring execution domains is unresolved. Candidates include block-level domains, function-level annotations, or return-type position markers. The syntax is NOT locked. The semantic requirements above are LOCKED regardless of which syntax is chosen.

### Domain Definitions

| Domain | Capabilities | Restrictions |
|--------|-------------|--------------|
| server | Filesystem, networking, databases, process spawning | No DOM, no browser APIs |
| client | DOM, fetch, localStorage, browser APIs | No filesystem, no process spawning |
| worker | Computation, message passing, limited I/O | No DOM, no direct system calls |
| native | Raw memory, system calls, FFI, hardware access | No automatic memory safety |

### Cross-Domain Communication

Cross-domain calls produce serialized messages. The compiler generates serialization code. Values crossing boundaries must satisfy a transmissibility constraint.

### AXIOM Differentiator: Compiler-Verified Typed Execution Boundaries

This is AXIOM's primary language-level differentiator. No mainstream language statically verifies that code operates within the capabilities of its declared execution domain.

In a typical full-stack application today (e.g., TypeScript + Express), nothing prevents server-only code (database queries, filesystem access) from accidentally ending up in a client bundle. The error is caught at runtime, or by a linter with limited understanding.

In AXIOM, the compiler verifies this at compile time. The compiler knows:

1. Which domain each function belongs to
2. Which domains each function calls
3. Which types cross domain boundaries
4. Whether those types are transmissible

This is verified at compile time, not runtime, not by a linter. It is a type-system property.

Why this is different from existing languages:

- TypeScript: No domain awareness. Client/server separation is manual.
- Rust: Ownership tracks memory, not execution domains.
- Go: No built-in client/server distinction.
- Swift: Actors track concurrency, not deployment domains.
- Kotlin: Coroutines track async, not deployment domains.

AXIOM tracks where code runs as a type-system property. This combination does not exist in any mainstream language.

---

## C. Memory Model

### Semantic Requirements (LOCKED)

1. Resources must be cleaned up predictably. The programmer must be able to determine, from source code, when a resource is released.
2. The type system must prevent accidental sharing of mutable state between concurrent computations without explicit synchronization.
3. The compiler should detect use-after-free and data race conditions at compile time when feasible.
4. The programmer must be able to express ownership transfer (a value is no longer available at its old location after being used elsewhere).
5. The programmer must be able to express borrowing (reading a value without taking ownership).
6. The language must provide a mechanism for values that outlive any single owner (dynamic allocation).
7. Different compilation targets may implement these semantics differently (GC, reference counting, manual management). The source-level semantics are target-independent.

### What the Programmer Expresses (PROPOSED)

The programmer expresses:

- **Ownership transfer**: When a value is passed to a function, it is consumed. This is the default.
- **Borrowing**: The programmer can create references to borrowed values. References can be shared (read-only) or exclusive (read-write).
- **Cleanup**: Values are cleaned up when they go out of scope. Types can define custom cleanup behavior.

The programmer does NOT express:

- How references are implemented (pointers, indices, handles)
- Whether memory is garbage-collected, reference-counted, or manually managed
- Lifetime annotation syntax (this is a source representation choice, not a semantic requirement)
- Stack vs heap placement (the backend decides)

### Proposed Source Representation (PROPOSED)

The operator symbols, keyword choices, and annotation positions are all PROPOSED. The semantic requirements (ownership transfer, borrowing, cleanup) are LOCKED.

### Backend Implementation (PROPOSED)

The source-level semantics are implemented differently per target:

| Target | Implementation Strategy | Rationale |
|--------|------------------------|-----------|
| VM (MVP) | Tracing garbage collector | Simple to implement, correct, no programmer burden |
| Native | Reference counting (initially) | Deterministic cleanup, no stop-the-world pauses |
| WASM | Linear memory + bump/region allocator | WASM has no native GC |

The backend choice is independent of the source-level semantics. The programmer writes the same code regardless of target.

### Why Not Locking a Specific Mechanism

The source-level semantics are expressed independently of the backend implementation. This is deliberate: the same AXIOM source code should compile to both a GC-managed VM and a reference-counted native binary without changing the source. Locking a specific backend mechanism in the language design would prevent this portability.

---

## D. Type System

### Semantic Requirements (LOCKED)

1. All types are checked at compile time.
2. The compiler infers types where possible; explicit annotations are required only when necessary for disambiguation.
3. Types are identified by name (nominal), not by structure.
4. The language supports sum types (a value is one of several variants) and product types (a value is a combination of several fields).
5. The absence of a value is represented in the type system (not by a null sentinel).
6. A function that can fail must declare that it can fail, and the failure must be handled or propagated by the caller.
7. The type system must be sound: a program that type-checks must not exhibit type errors at runtime.

### What the Programmer Expresses (PROPOSED)

**Primitives:** Integer types (at least 32-bit and 64-bit), floating-point types, boolean, character, string.

**Compound types:** Tuples, structs, enums (sum types with named variants, variants may carry data), arrays, lists.

**Function types:** Functions have declared parameter types and return types. Higher-order functions are supported.

### Absence Representation (LOCKED requirement, PROPOSED syntax)

The type system must distinguish between a value being present and a value being absent. The exact type name and syntax are PROPOSED. The semantic requirement is LOCKED.

### Failure Representation (LOCKED requirement, PROPOSED syntax)

A function that can fail must declare the failure in its return type. The caller must handle or propagate the failure. The exact type name and propagation syntax are PROPOSED. The semantic requirement is LOCKED.

### Why No Exceptions

Exceptions violate AXIOM's explicit-over-implicit principle: invisible control flow, type erasure at boundaries, resource leaks, and the compiler cannot verify that errors are handled. A failure type in the return signature satisfies all locked requirements.

### Why Nominal Typing

Structural typing creates implicit coupling. Changing a field name silently breaks code that uses the type structurally. Nominal typing means a type satisfies an interface only if it explicitly declares it. More verbose but more predictable.

### Generics (PROPOSED, deferred from MVP)

Parametric polymorphism with type constraints. Syntax and constraint mechanism are PROPOSED. Deferred from MVP.

### Traits / Interfaces (OPEN)

The mechanism for defining shared behavior across types is an OPEN research question. Options include Rust-style traits, Go-style interfaces, or Swift-style protocols. The choice affects dynamic dispatch, generic constraints, and the relationship between enums and shared behavior.

---

## E. Concurrency Model

### Semantic Requirements (LOCKED)

1. The language must support concurrent execution of multiple computations.
2. The programmer must be able to express that a computation runs concurrently with the current flow.
3. The programmer must be able to wait for a concurrent computation to produce a result.
4. Communication between concurrent computations must be explicit (not shared memory by default).
5. The type system must prevent data races at compile time when feasible.
6. Concurrent computations must be cancellable when their result is no longer needed.

### Proposed Representation (PROPOSED)

- Lightweight tasks spawned with a concurrency primitive
- Message channels for inter-task communication
- A waiting mechanism for receiving results from concurrent tasks

The exact syntax is PROPOSED. The semantic requirements are LOCKED.

### Why Not Threads

OS threads are heavy (1MB+ stack, expensive context switching). AXIOM tasks are lightweight, scheduled by the runtime. On native targets, the runtime maps tasks to a thread pool. On WASM, tasks map to the event loop.

### Why Not Actor Model

The actor model creates complexity in type tracking and message schema evolution. AXIOM uses structured concurrency with channels as the primitive. Actors can be built on top.

### Task Lifecycle (PROPOSED)

Tasks have a lifecycle: running, completed, cancelled, failed. When the parent scope exits, child tasks are cancelled or awaited. This is the structured concurrency constraint (LOCKED).

---

## F. Compiler Pipeline

### Stage 1: Lexer

Input: Source text. Output: Token stream. Hand-written scanner. Tokens carry source location.

### Stage 2: Parser

Input: Token stream. Output: Untyped AST. Recursive descent parser. Preserves all source information.

### Stage 3: Name Resolution

Input: Untyped AST. Output: Resolved AST (names bound to declarations). Handles module imports, scope resolution, method resolution.

### Stage 4: Type Checking

Input: Resolved AST. Output: Typed AST. Type inference, validation, overload resolution. Reports type errors with source locations.

### Stage 5: Domain Analysis (LOCKED stage, PROPOSED details)

Input: Typed AST. Output: Verified AST.

This stage is AXIOM's differentiator. It verifies:

- Each function operates only within its declared domain's capabilities
- Cross-domain calls are properly routed through the message protocol
- Values crossing domain boundaries are transmissible
- No domain-forbidden operations are used

This stage does not exist in other languages. It is the compiler-verified typed execution boundary.

### Stage 6: Lowering to HIR

Input: Verified AST. Output: HIR. Desugars pattern matching, loops, closures. Preserves type information.

### Stage 7: Optimization

Input: HIR. Output: Optimized HIR. Constant folding, dead code elimination, inlining. Conservative, correctness over performance.

### Stage 8: Bytecode Emission (LOCKED for MVP)

Input: Optimized HIR. Output: VM bytecode. Stack-based bytecode for the AXIOM VM.

---

## G. Why Bytecode VM for the MVP

### The Contradiction Resolved

The previous architecture stated "interpret HIR directly" as the MVP target but also defined a "stack-based VM" as the runtime. These are contradictory: either you interpret the HIR tree directly (tree-walk interpreter) or you emit bytecode and run a VM.

### Decision: Bytecode VM (LOCKED)

The MVP uses a bytecode VM, not a tree-walk HIR interpreter.

### Rationale

1. **Bytecode is simpler to generate from HIR** than machine code but produces a representation that is more efficient to execute than tree-walk interpretation.
2. **The VM provides a clean target for future optimization.** A tree-walk interpreter cannot be optimized without rewriting it. A bytecode VM can have its instruction set extended, its interpreter optimized, or be replaced by a JIT later.
3. **Bytecode validates the full compiler pipeline.** If the compiler can emit correct bytecode, it demonstrates the frontend, type checker, and lowering stages all work. A tree-walk interpreter skips the lowering and code generation stages entirely.
4. **Bytecode is the natural intermediate step toward native and WASM targets.** Both native code generation and WASM emission benefit from having a well-defined IR. The bytecode format serves as that IR for the VM target.
5. **Execution speed is adequate for development.** A bytecode VM is typically 10-100x faster than tree-walk interpretation. This makes AXIOM usable for real programs during development, not just toy examples.

### Why Not Tree-Walk Interpretation

Tree-walk interpretation is faster to implement but creates a dead end: the code generation stage is never exercised, optimization is impossible, and the representation is too high-level for future backend work. The small additional effort to emit bytecode pays for itself immediately and prevents architectural debt.

### Why Not Native Code for the MVP

Native code generation (via LLVM or a custom code generator) is significantly more complex. The MVP needs a working compile-and-run cycle. The bytecode VM provides this with minimal implementation cost.

### HIR is Internal, Bytecode is the Output

The HIR is an internal compiler representation used for optimization and type-directed lowering. It is never directly executed. The compiler lowers HIR to bytecode. This is a clean separation:

```
Source -> AST -> Resolved AST -> Typed AST -> Verified AST -> HIR -> Bytecode
```

---

## H. Runtime Architecture

### Runtime Components (LOCKED requirements, PROPOSED implementation)

1. **Task Scheduler** - Manages lightweight tasks, context switching, work stealing
2. **Garbage Collector** - Tracing GC for the VM target (implementation varies by target)
3. **Channel Implementation** - Typed message channels for inter-task communication
4. **I/O Reactor** - Event loop for async I/O
5. **Memory Allocator** - Configurable allocator interface

### Runtime Minimalism (LOCKED)

The runtime must be small enough to link into every AXIOM program without significant overhead. Target: under 200KB compiled for the VM target.

### Host Functions (PROPOSED)

The runtime exposes operations to AXIOM code through host functions. These are the primitive operations: I/O, networking, task management, channel operations, time, and system information. Host functions are the boundary between AXIOM code and the runtime.

---

## I. Multi-Target Strategy

### Target Architecture (LOCKED architecture, PROPOSED timeline)

```
HIR
  -> VM Backend (MVP): HIR -> Bytecode
  -> Native Backend (future): HIR -> LLVM IR -> Machine code
  -> WASM Backend (future): HIR -> WASM bytecode
```

The frontend (source -> HIR) is shared. Only the backend differs per target.

### Why LLVM for Native (PROPOSED)

LLVM handles register allocation, instruction selection, and optimization. It is the proven path for native code generation. However, LLVM is a heavy dependency. The native backend is deferred from the MVP.

### Target-Specific Differences

| Feature | Native | WASM | VM |
|---------|--------|------|----|
| Memory management | Reference counting | Linear memory + allocator | Tracing GC |
| Threading | OS threads + green tasks | Web Workers | Green tasks |
| I/O | OS syscalls | Browser APIs / WASI | Host functions |

---

## J. Unresolved Research Questions (OPEN)

1. **Exact syntax** - Syntax should follow semantics, not lead them. Deferred.
2. **Module system semantics** - File-based vs directory-based vs hybrid.
3. **Trait / interface system** - Rust traits, Go interfaces, Swift protocols, or a new design.
4. **Absence syntax** - Exact type name and operator sugar.
5. **Failure syntax** - Exact type name and propagation operator.
6. **Ownership syntax** - Exact symbols and annotation positions.
7. **String encoding** - UTF-8 is the leading candidate but not locked.
8. **Numeric type coercion** - Explicit conversion vs limited implicit coercion.
9. **Compile-time execution** - Whether AXIOM supports comptime evaluation.
10. **Macro system** - Whether AXIOM has macros and what kind.
11. **Cross-domain serialization format** - JSON, binary, or custom protocol.
12. **How transmissibility is defined** - Which types can cross domain boundaries.

---

## K. Decision Registry

### LOCKED Decisions

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| L1 | Implementation language | Rust | Memory safety, good tooling, compiles fast, no runtime dependency |
| L2 | IR layers | AST -> Resolved -> Typed -> Verified -> HIR -> Bytecode | Each layer has clear responsibility; domain analysis is a distinct stage |
| L3 | MVP target | Bytecode VM | Validates full pipeline; extensible; adequate speed for development |
| L4 | Compiler output | Stack-based bytecode | Simple to generate, simple to interpret, compact |
| L5 | Type system | Nominal, static, sound | Predictable; refactoring-safe; catches errors at compile time |
| L6 | Absence | Type-system enforced, no null sentinel | Eliminates null-related runtime errors |
| L7 | Failure | Declared in return type, no exceptions | Explicit, composable, compiler-verifiable |
| L8 | Concurrency | Structured tasks with message passing | Cancellable, type-safe, no hidden data races |
| L9 | Domain verification | Compile-time, not runtime or linter | The core differentiator; must be sound |
| L10 | Source-level memory semantics | Ownership, borrowing, cleanup | Target-independent; backend implements per target |

### PROPOSED Decisions

| # | Decision | Choice | Status |
|---|----------|--------|--------|
| P1 | Ownership syntax | Consumed by default; borrowing via annotation | May change during syntax design |
| P2 | Absence type | Sum type with present/absent variants | Name and syntax TBD |
| P3 | Failure type | Sum type with success/failure variants | Name and syntax TBD |
| P4 | Domain syntax | Undecided (annotations, blocks, or position markers) | Candidate syntax for exploration |
| P5 | String encoding | UTF-8, immutable by default | Leading candidate |
| P6 | Numeric types | Fixed-width integers, IEEE 754 floats | No implicit coercion |
| P7 | Module system | File-based modules | Simple; may evolve |
| P8 | Generics | Parametric polymorphism with constraints | Deferred from MVP |
| P9 | VM GC | Tracing garbage collector | Simplest correct implementation |
| P10 | Native memory | Reference counting | Deterministic cleanup |

### OPEN Research Questions

| # | Question | Options |
|---|----------|---------|
| O1 | Exact syntax for all constructs | Multiple candidates |
| O2 | Trait / interface system | Traits, interfaces, protocols, or new |
| O3 | Cross-domain serialization format | JSON, binary, custom |
| O4 | Transmissibility definition | Which types cross boundaries |
| O5 | Compile-time execution | comptime, macros, or neither |
| O6 | Macro system | Declarative, procedural, or none |
| O7 | String encoding final choice | UTF-8, UTF-16, or rope |
| O8 | Numeric coercion rules | Explicit-only vs limited implicit |
| O9 | Lifetime annotation syntax | If needed, what form |
| O10 | Dynamic dispatch mechanism | Vtable, dictionary, or other |
