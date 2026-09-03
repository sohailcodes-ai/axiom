# AXIOM Targets (Revised)

## Overview

AXIOM compiles to multiple targets. Each target has different constraints and capabilities. The compiler's HIR is lowered differently for each target. The frontend (source -> HIR) is shared across all targets.

## Target Architecture

```
AXIOM Source
    |
    v
+-------------------+
| Frontend (shared)  | -> HIR
+-------------------+
    |
    +---> VM Backend (MVP): HIR -> Bytecode
    +---> Native Backend (future): HIR -> LLVM IR -> Machine code
    +---> WASM Backend (future): HIR -> WASM bytecode
```

## Target: AXIOM VM (LOCKED for MVP)

Status: MVP target
Output: .axvm bytecode file
Execution: axiom run program.axvm

### Characteristics

- Stack-based bytecode
- Tracing garbage collector
- Green task scheduler (structured concurrency)
- Standard library via host functions
- No external dependencies required

### Why the VM First

The VM is the fastest path to a working compile-and-run cycle:

1. No code generation complexity (no machine code, no linker)
2. No platform-specific code
3. Fast compilation
4. Good error reporting (bytecode position maps to source location)
5. The bytecode format validates the full compiler pipeline

The VM does not mean "slow for development." A bytecode VM is typically 10-100x faster than tree-walk interpretation, making AXIOM usable for real programs during development.

### VM Architecture

```
.axvm file
  Header (magic, version, checksum)
  Constant pool
  Type declarations
  Function table
    For each function:
      Name
      Parameter types
      Return type
      Local variable types
      Bytecode instructions
  Entry point
```

### VM Runtime

The VM runtime is an interpreter loop:

1. Fetch the next instruction
2. Decode the instruction
3. Execute the instruction (push/pop/call/jump)
4. Repeat until halt

The runtime includes:

- Stack management (operand stack, call stack)
- Local variable storage
- Garbage collector (tracing GC for heap values)
- Host function interface (I/O, networking, tasks)
- Task scheduler (green tasks, channels)

### Why Not Tree-Walk Interpretation

Tree-walk interpretation is faster to implement but:

1. Does not validate the code generation stage (HIR -> bytecode is skipped)
2. Cannot be optimized without rewriting the interpreter
3. Is 10-100x slower than bytecode interpretation
4. Creates a dead end for future optimization (JIT, native codegen)

The bytecode VM validates the full pipeline and provides a foundation for future work.

## Target: Native (DEFERRED)

Status: Deferred (post-MVP)
Output: ELF (Linux), Mach-O (macOS), PE (Windows) executable
Execution: Direct OS execution

### Characteristics

- Reference counting for memory management (or optional tracing GC)
- OS thread pool for task scheduling
- Direct system calls for I/O
- C ABI for FFI
- Platform-specific optimizations

### Implementation Path

1. Generate LLVM IR from HIR
2. Use LLVM to generate machine code
3. Use system linker to produce executable

Alternative (simpler): Use a tiny custom code generator that targets x86-64/ARM64 directly. This avoids the LLVM dependency but limits optimization.

### Why LLVM

LLVM handles register allocation, instruction selection, and optimization. It is the proven path for native code generation. The LLVM dependency is significant but justified for the quality of output.

## Target: WASM (DEFERRED)

Status: Deferred (post-MVP)
Output: .wasm module
Execution: Browser, WASI runtime, or WASM VM

### Characteristics

- WASM linear memory for heap
- No built-in GC (uses bump allocation or WASM-GC proposal)
- Web Workers for concurrency (browser) or WASI threads
- Browser APIs via WASM imports
- WASI for filesystem/networking (non-browser)

### Implementation Path

1. Generate WASM bytecode from HIR
2. Use wasm-tools or wasm-encoder crate for WASM generation
3. Package with JavaScript glue for browser targets

### WASM-Specific Considerations

- WASM has no native string type (strings are byte arrays in linear memory)
- WASM has no native GC (reference counting or bump allocation)
- WASM has no native threading (WASM threads proposal or message passing)
- WASM imports must be declared (the compiler generates an import section)

## Cross-Target Consistency (LOCKED)

The same AXIOM source code must produce semantically equivalent results across targets. Differences are in implementation, not semantics:

| Concern | VM | Native | WASM |
|---------|-----|--------|------|
| Memory management | Tracing GC | Reference counting | Linear memory + allocator |
| Concurrency | Green tasks | OS threads + green tasks | Web Workers / WASI threads |
| I/O | Host functions | OS syscalls | WASM imports |
| Performance | Interpreted (adequate) | Compiled (fast) | Compiled (medium) |
| Startup time | Fast | Medium | Fast |
| Binary size | Small | Large | Medium |

## Target Selection (PROPOSED)

The target is specified at compile time:

```
axiom build --target vm program.ax
axiom build --target native program.ax
axiom build --target wasm program.ax
```

Default target: VM (for development speed).
