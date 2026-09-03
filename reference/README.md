# AXIOM Reference Implementation

This directory contains the Python prototype of the AXIOM compiler and VM.

**IMPORTANT**: This is NOT the actual AXIOM implementation. It is a reference implementation used for language design validation.

The actual AXIOM compiler and runtime are implemented in Rust (see `../compiler/`).

## Purpose

This Python prototype was used to:

1. Validate the AXIOM language syntax
2. Test the compiler pipeline design
3. Verify that the VM bytecode design works
4. Provide a working example for the Rust implementation

## Files

- `main.py` - CLI entry point
- `lexer.py` - Tokenizer
- `tokens.py` - Token definitions
- `parser.py` - Parser
- `ast_nodes.py` - AST node definitions
- `name_resolver.py` - Name resolution
- `type_checker.py` - Type checking
- `hir.py` - HIR definitions
- `hir_lower.py` - HIR lowering
- `codegen.py` - Bytecode generation
- `bytecode.py` - Bytecode definitions
- `vm.py` - Virtual machine
- `errors.py` - Error types

## Usage

```bash
# Run a program
python main.py run ../examples/hello.ax

# Tokenize
python main.py lex ../examples/hello.ax

# Parse
python main.py parse ../examples/hello.ax
```

## Status

This reference implementation is complete and functional. It demonstrates that the AXIOM language design works correctly.

The Rust implementation should produce equivalent behavior.
