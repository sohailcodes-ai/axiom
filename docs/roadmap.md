# AXIOM Roadmap (Revised)

## Decision Classification Legend

- **LOCKED** - Final decision. Will not change without documented justification.
- **PROPOSED** - Best current direction. May change based on evidence.
- **OPEN** - Research question. Requires further investigation.

---

## Phase 0: Architecture (Current)

**Goal:** Define the language design and architecture before writing code.

**Status:** In progress (revised)

**Deliverables:**

- [x] Architecture overview (revised)
- [x] Execution model (semantic requirements locked, syntax proposed)
- [x] Type system design (semantic requirements locked, syntax proposed)
- [x] Memory model (semantic requirements locked, backend strategy proposed)
- [x] Concurrency model (semantic requirements locked, syntax proposed)
- [x] Error handling (semantic requirements locked, syntax proposed)
- [x] Compiler architecture (pipeline locked, stages defined)
- [x] IR design (levels defined, HIR/bytecode separation clarified)
- [x] Target strategy (VM locked for MVP, native/WASM deferred)
- [ ] Syntax design (deferred to Phase 1)
- [ ] Standard library design (deferred to Phase 2)

---

## Phase 1: MVP Compiler

**Goal:** A working compiler that can parse, type-check, and run AXIOM programs through the bytecode VM.

**Estimated effort:** 2-4 weeks for a solo developer

**Deliverables:**

- Lexer (tokenizer)
- Parser (recursive descent)
- AST data structures
- Name resolver
- Type checker (basic)
- Domain analyzer (basic - single domain for MVP)
- HIR data structures
- Bytecode emitter
- Bytecode VM (interpreter)
- CLI tool (axiom run, axiom build)
- Basic error reporting

**Language features (MVP):**

- Modules (basic import/export)
- Primitive types (Int, Float, Bool, String)
- Functions (with return types)
- Structs
- Enums (basic, no data in variants initially)
- Let bindings (immutable by default)
- If/else expressions
- While loops
- Pattern matching (basic - match on enum variants)
- Print / basic I/O
- Absence handling (basic - sum type for present/absent)
- Failure handling (basic - sum type for success/failure, propagation operator)

**Deferred from MVP:**

- Generics
- Ownership/borrowing enforcement (use GC only)
- Concurrency (single-task execution)
- Multiple execution domains (single domain for MVP)
- Multiple targets (VM only)
- Package manager
- Standard library beyond basic I/O

**MVP Differentiator:**

The MVP includes a basic domain analysis stage. Even in single-domain mode, the architecture for domain verification is in place. This validates the compiler pipeline including AXIOM's differentiator.

**Success criteria:**

- Can write an AXIOM program
- Can compile it to bytecode without errors
- Can run the bytecode and see output
- Error messages are useful
- The domain analysis stage is implemented (even if simplified for MVP)

---

## Phase 2: Core Language

**Goal:** A usable language with essential features.

**Estimated effort:** 1-2 months

**Deliverables:**

- Generics (basic)
- Pattern matching (full - data in variants)
- Absence handling (full)
- Failure handling (full - propagation, conversion, accumulation)
- Closures
- Iterators
- Ownership semantics (basic enforcement)
- Standard library (basic)
  - String operations
  - List/Array operations
  - Map operations
  - File I/O
  - HTTP client/server (basic)
  - JSON parsing
- Package manager (basic)
- Test runner

---

## Phase 3: Concurrency

**Goal:** Full concurrency support.

**Estimated effort:** 1 month

**Deliverables:**

- Task spawning
- Channels (typed)
- Async/await (if chosen over channel-only model)
- Atomic operations
- Mutex/RwLock
- Task cancellation
- Structured concurrency enforcement

---

## Phase 4: Execution Domains

**Goal:** Full-stack support with compiler-verified typed execution boundaries.

**Estimated effort:** 1-2 months

**Deliverables:**

- Domain declarations (source syntax finalized)
- Domain analysis (full - all domain constraints verified)
- Cross-domain serialization
- Client compilation unit
- Server compilation unit
- Worker compilation unit
- Browser DOM bindings (for client target)
- HTTP server bindings (for server target)

This phase implements AXIOM's primary differentiator.

---

## Phase 5: Native Target

**Goal:** Compile to native executables.

**Estimated effort:** 2-3 months

**Deliverables:**

- LLVM IR generation (or custom code generator)
- Native code generation (x86-64, ARM64)
- Linker integration
- Native runtime (reference counting, thread pool)
- FFI interface
- Cross-compilation support

---

## Phase 6: WASM Target

**Goal:** Compile to WebAssembly.

**Estimated effort:** 1-2 months

**Deliverables:**

- WASM bytecode generation
- WASM module packaging
- JavaScript glue code (for browser)
- WASI integration (for non-browser)
- Browser API bindings
- WASM runtime

---

## Phase 7: Ecosystem

**Goal:** Tooling and ecosystem.

**Estimated effort:** Ongoing

**Deliverables:**

- Package registry
- Package manager (full)
- Formatter
- Language server (LSP)
- Debugger
- Profiler
- Documentation generator
- IDE extensions
- Bootstrap (AXIOM compiler written in AXIOM)

---

## Phase 8: Advanced Features

**Goal:** Advanced language features.

**Estimated effort:** Ongoing

**Features:**

- Compile-time execution (comptime)
- Macros (declarative)
- Effect system
- Region-based allocation
- SIMD/vectorization
- Interoperability (C, Rust, Python bindings)

---

## Milestone Dates

| Phase | Target Date | Status |
|-------|-------------|--------|
| Phase 0 | Now | In progress (revised) |
| Phase 1 | +4 weeks | Not started |
| Phase 2 | +3 months | Not started |
| Phase 3 | +4 months | Not started |
| Phase 4 | +6 months | Not started |
| Phase 5 | +8 months | Not started |
| Phase 6 | +10 months | Not started |
| Phase 7 | +12 months | Not started |
| Phase 8 | +18 months | Not started |

These dates are estimates for a solo developer. Actual timeline depends on available time and complexity encountered.

---

## Success Criteria Summary

| Phase | Key Criterion |
|-------|---------------|
| Phase 1 | Can compile and run a basic AXIOM program end-to-end via bytecode VM |
| Phase 2 | Can write a REST API server or CLI tool with error handling |
| Phase 3 | Can run concurrent tasks with typed channels |
| Phase 4 | Can write a full-stack app with compiler-verified domain boundaries |
| Phase 5 | Can compile to native executable with reasonable performance |
| Phase 6 | Can compile to WASM and run in a browser |
| Phase 7 | Can install packages, get IDE support, debug programs |
| Phase 8 | Can compile AXIOM with AXIOM, use compile-time features |
