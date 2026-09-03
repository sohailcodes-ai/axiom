# AXIOM Compiler MVP — Implementation Plan

## Goal

Get `axiom run hello.ax` working end-to-end:

1. Read AXIOM source file
2. Lex it into tokens
3. Parse tokens into AST
4. Resolve names
5. Type-check
6. Lower to HIR
7. Lower HIR to bytecode
8. Execute bytecode in VM
9. Produce output

## Implementation Language

Python 3.13 (available on the system). The compiler will be rewritten in Rust for the bootstrap (Phase 7).

## Directory Structure

```
compiler/
├── src/
│   ├── __init__.py
│   ├── main.py           # CLI entry point
│   ├── lexer.py          # Tokenizer
│   ├── tokens.py         # Token types
│   ├── parser.py         # Recursive descent parser
│   ├── ast_nodes.py      # AST node definitions
│   ├── name_resolver.py  # Name resolution
│   ├── type_checker.py   # Type checking
│   ├── hir.py            # HIR data structures
│   ├── hir_lower.py      # AST to HIR lowering
│   ├── codegen.py        # HIR to bytecode
│   ├── bytecode.py       # Bytecode definitions
│   ├── vm.py             # Bytecode interpreter
│   └── errors.py         # Error types
├── examples/
│   └── hello.ax           # First test program
└── tests/
    └── test_lexer.py      # Lexer tests
```

## Milestones

### Milestone 1: Lexer (tokens + lexer.py)
- Tokenize keywords, identifiers, literals, operators, delimiters
- Track source locations (line, column)
- Handle comments and whitespace

### Milestone 2: Parser (parser.py + ast_nodes.py)
- Parse declarations: structs, enums, functions, imports
- Parse expressions: literals, binary ops, function calls, field access
- Parse statements: let, if, while, for, return, match
- Parse types: primitives, named types, function types, maybe, failure

### Milestone 3: Name Resolution (name_resolver.py)
- Bind names to declarations
- Resolve module imports
- Resolve field access on structs
- Detect undefined variables

### Milestone 4: Type Checking (type_checker.py)
- Infer types for expressions
- Check type consistency
- Check function call arguments
- Check return types
- Check match exhaustiveness (basic)

### Milestone 5: HIR Lowering (hir_lower.py + hir.py)
- Lower AST to HIR
- Desugar pattern matching
- Lower control flow
- Lower closures (basic)

### Milestone 6: Code Generation (codegen.py + bytecode.py)
- Generate bytecode from HIR
- Handle function calls
- Handle local variables
- Handle control flow (if, while, break, continue)
- Handle pattern matching (basic)

### Milestone 7: VM (vm.py)
- Stack-based bytecode interpreter
- Handle arithmetic, comparison, logical ops
- Handle function calls and returns
- Handle local variables
- Handle heap allocation (basic GC)
- Handle built-in functions (print, etc.)

### Milestone 8: Integration (main.py)
- CLI: `axiom run file.ax`
- Wire together all stages
- Error reporting with source locations

## MVP Language Subset

### Supported
- Primitive types (i64, f64, bool, String)
- Structs (definition, construction, field access)
- Enums (definition, construction, pattern matching)
- Functions (declaration, calls, return)
- Variables (let, let mut, reassignment)
- Arithmetic (+, -, *, /, %)
- Comparisons (==, !=, <, >, <=, >=)
- Boolean logic (&&, ||, !)
- If/else expressions
- While loops
- For loops (basic)
- Blocks (as expressions)
- Match expressions (basic)
- maybe T (Some, None, pattern matching)
- Failure return types (-> T ! E)
- ? propagation
- Import statements (basic)
- Module paths (::)
- &T / &mut T (syntax only, GC-backed)
- Print function (built-in)

### Deferred
- Closures (lambda syntax only)
- Generics
- Impl blocks (methods)
- Execution domains
- Concurrency (spawn, await, chan)
- Named arguments
- Struct update syntax
- String interpolation
- Advanced pattern matching
- Module system (re-exports, visibility)

## Test Program

```axiom
fn main() {
    print("Hello, World!")
}
```

This must compile and execute successfully.
