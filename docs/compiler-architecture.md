# AXIOM Compiler Architecture (Revised)

## Overview

The AXIOM compiler is a multi-stage pipeline. Each stage consumes and produces a distinct data structure. No stage reaches back into a prior representation.

This design enables:

- Independent testing of each stage
- Swappable backends
- Incremental compilation
- Clear error reporting at each stage

## Pipeline

```
Source Code (text)
    |
    v
+---------+
|  Lexer   | -> Token stream
+---------+
    |
    v
+---------+
|  Parser  | -> Untyped AST
+---------+
    |
    v
+------------------+
| Name Resolution   | -> Resolved AST (names bound to declarations)
+------------------+
    |
    v
+------------------+
| Type Checking     | -> Typed AST (types attached to all expressions)
+------------------+
    |
    v
+-------------------+
| Domain Analysis    | -> Verified AST (execution boundaries checked)
+-------------------+
    |
    v
+-------------+
|  Lowering    | -> HIR (High-level IR)
+-------------+
    |
    v
+--------------+
| Optimization | -> Optimized HIR
+--------------+
    |
    v
+-------------------+
| Bytecode Emission  | -> VM bytecode
+-------------------+
    |
    v
+-------------------+
| [Future backends]  | -> Native code, WASM
+-------------------+
```

## Stage Details

### Stage 1: Lexer

Input: Source text (string)
Output: Vec of tokens

Each token carries:

- Token kind (keyword, identifier, literal, operator, delimiter)
- Source location (file, line, column)
- Raw text

The lexer is a hand-written scanner. No lexer generator is used - this avoids external dependencies and gives full control over error messages.

### Stage 2: Parser

Input: Token stream
Output: Untyped AST (Concrete Syntax Tree)

The parser is a recursive descent parser with precedence climbing for expressions. It produces an untyped AST that preserves all source information (whitespace, comments, parenthesization).

The AST is an algebraic data type representing all syntactic forms of the AXIOM language.

### Stage 3: Name Resolution

Input: Untyped AST
Output: Resolved AST

Name resolution binds names to their declarations. It resolves:

- Module imports
- Local variable references
- Function references
- Type references
- Field access on structs

The output carries declaration IDs instead of names. This eliminates name-based lookup in later stages.

### Stage 4: Type Checking

Input: Resolved AST
Output: Typed AST

Type checking assigns types to every expression. It performs:

- Type inference
- Type validation
- Overload resolution

The output carries type information on every expression node.

### Stage 5: Domain Analysis (LOCKED stage)

Input: Typed AST
Output: Verified AST

This stage is AXIOM's differentiator. It verifies execution boundary constraints.

**What it checks:**

1. Each function operates only within its declared domain's capabilities
2. Cross-domain calls are properly routed through the message protocol
3. Values crossing domain boundaries are transmissible
4. No domain-forbidden operations are used

**How it works:**

Each domain has a capability set. The type system tracks which domain each function belongs to. When a function call is resolved, the compiler checks:

- Is the called function in the same domain? If so, direct call.
- Is the called function in a different domain? If so, it must be a cross-domain call, and the arguments must be transmissible.

**Why it is a separate stage:**

Domain analysis depends on type checking (it needs type information to verify transmissibility). It does not depend on the IR representation. Making it a separate stage keeps the type checker focused on type correctness and the domain analyzer focused on domain correctness.

**Why it is not a linter:**

A linter runs after compilation and can be skipped. Domain analysis is a compiler stage - it runs as part of every compilation and its results affect code generation. This is the difference between "the compiler rejects invalid code" and "an optional tool warns about suspicious code."

### Stage 6: Lowering to HIR

Input: Verified AST
Output: HIR

The HIR is a simplified representation that preserves type information but removes syntactic sugar:

- Pattern matching lowered to decision trees or nested if/else
- Loops become explicit control flow
- Closures become function pointers + captured environment structs
- String concatenation becomes explicit allocation + copy
- Operator overloading becomes explicit function calls

The HIR is still typed and still carries source locations.

### Stage 7: Optimization

Input: HIR
Output: Optimized HIR

Optimizations are conservative and correct. No optimization may change program semantics.

**MVP optimizations:**

- Constant folding
- Dead code elimination
- Inlining (small functions)
- Algebraic simplification (x + 0 = x, x * 1 = x)

**Future optimizations:**

- Function specialization (generics monomorphization)
- Loop optimization
- Escape analysis (stack allocation of heap values)
- Vectorization hints

### Stage 8: Bytecode Emission (LOCKED for MVP)

Input: Optimized HIR
Output: VM bytecode

The HIR is lowered to stack-based bytecode. Each function becomes a sequence of bytecode instructions. The bytecode includes:

- Type information (for runtime type checking)
- Source location mapping (for error reporting and debugging)
- Constant pool (for literals)
- Function table (for calls)

## Why HIR Before Bytecode

The HIR serves as the bridge between the high-level typed AST and the low-level bytecode. It:

1. Preserves type information for type-directed lowering
2. Removes syntactic sugar so the bytecode emitter does not need to handle it
3. Enables optimization before code generation
4. Provides a representation that can be shared across backends (VM, native, WASM)

Without HIR, the bytecode emitter would need to handle pattern matching, closures, operator overloading, and other high-level constructs directly. This makes the emitter complex and error-prone.

## Error Handling in the Compiler (LOCKED)

Each stage produces errors with source locations. Errors are collected, not immediately fatal. The compiler reports all errors found in a stage before moving to the next.

Error format:

- Error code (for documentation lookup)
- Source location (file, line, column)
- Description of the error
- Source context (the relevant line of code)
- Suggestion for fixing (when possible)

## Incremental Compilation (PROPOSED)

The compiler supports incremental compilation:

- Each module is compiled independently
- Only changed modules are recompiled
- Type information is cached between compilations
- The dependency graph is tracked

This is not in the MVP but the architecture supports it.

## Compiler Implementation (LOCKED)

The compiler is written in Rust. Key crates:

| Crate | Purpose |
|-------|---------|
| axiom-lexer | Tokenizer |
| axiom-parser | Parser, AST |
| axiom-name-res | Name resolution |
| axiom-types | Type system |
| axiom-domain | Domain analysis |
| axiom-hir | HIR data structures |
| axiom-codegen | Bytecode emission |
| axiom-vm | VM runtime |
| axiom-cli | CLI interface |

Separate crates enable:

- Independent testing
- Clear dependency boundaries
- Potential reuse (e.g., using axiom-parser in tooling)

## Bytecode Format (PROPOSED)

Stack-based bytecode:

```
Instruction set:
  Push(value)         - Push a constant onto the stack
  Pop                 - Discard top of stack
  LocalGet(id)        - Push a local variable onto the stack
  LocalSet(id)        - Pop top of stack into a local variable
  Call(fn_id, argc)   - Call a function
  Return              - Return from current function
  Jump(target)        - Unconditional jump
  ConditionalJump(t)  - Pop and jump if false
  Add, Sub, Mul, Div  - Arithmetic
  Equal, Less, Greater - Comparison
  Construct(type_id)  - Construct a struct
  FieldGet(field_id)  - Get a struct field
  FieldSet(field_id)  - Set a struct field
  ...additional instructions as needed
```

Each function is compiled to a sequence of instructions with local variable slots. The VM interprets these instructions.
