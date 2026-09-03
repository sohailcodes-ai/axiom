# AXIOM IR Design (Revised)

## Overview

AXIOM uses a multi-level IR approach. Each level serves a specific purpose and has specific properties. The key design decision: the HIR is an internal compiler representation, never directly executed. The compiler lowers HIR to bytecode for the VM target.

## IR Levels

### Level 1: Untyped AST

Purpose: Preserve all source information
Typed: No
Used by: Parser -> Name Resolution

The AST is a direct mapping of the source code structure. It preserves all tokens, parenthesization, operator precedence, and source locations.

### Level 2: Resolved AST

Purpose: Bind names to declarations
Typed: No (names resolved, types not yet checked)
Used by: Name Resolution -> Type Checking

Names are replaced with declaration IDs. The symbol table is integrated.

### Level 3: Typed AST

Purpose: Carry type information
Typed: Yes
Used by: Type Checking -> Domain Analysis -> Lowering

Every expression has a type. Generic types are instantiated (when generics are implemented). Type errors are reported.

### Level 4: Verified AST

Purpose: Verify execution domain boundaries
Typed: Yes (inherited from Typed AST)
Used by: Domain Analysis -> Lowering

This is AXIOM's unique IR level. It carries domain membership information and transmissibility verification. It does not exist in other language compilers.

Properties:

- Each function has a verified domain membership
- Cross-domain calls are verified and annotated
- Transmissible types are verified
- Domain-forbidden operations are flagged

### Level 5: HIR (High-level IR)

Purpose: Simplified representation for optimization and code generation
Typed: Yes (types preserved from Typed AST)
Used by: Lowering -> Optimization -> Bytecode Emission

HIR differs from AST:

- Pattern matching lowered to decision trees
- Loops are explicit control flow
- Closures are explicit (captured variables structured)
- Operator overloading resolved
- String operations explicit
- All syntactic sugar removed

HIR properties:

- Typed (every node has a type)
- Located (every node has a source location)
- Flat-ish (deeply nested expressions broken into statements with temporaries)
- Structurable (can be converted back to structured control flow for WASM)

### Level 6: Bytecode (MVP output)

Purpose: Executable representation for the VM
Typed: Yes (type information encoded in bytecode)
Used by: Bytecode Emission -> VM execution

Stack-based bytecode. Each function becomes a sequence of instructions with local variable slots.

## Why HIR Before Bytecode

The HIR serves as the bridge between the high-level typed AST and the low-level bytecode:

1. **Preserves type information** for type-directed lowering
2. **Removes syntactic sugar** so the bytecode emitter does not need to handle pattern matching, closures, operator overloading
3. **Enables optimization** before code generation
4. **Provides a shared representation** across backends (VM bytecode can be generated from HIR; native code generation can also use HIR)

Without HIR, the bytecode emitter would need to handle all high-level constructs directly, making it complex and error-prone.

## Why Not SSA Directly

SSA (Static Single Assignment) is excellent for optimization but:

- Requires phi nodes for control flow merge points
- Loses high-level type structure (everything becomes register operations)
- Makes type-directed code generation harder
- Is harder to read and debug when generated from high-level source

HIR preserves the type structure while being simple enough for optimization. SSA can be generated from HIR in backends that benefit from it (like the LLVM backend for native code).

## HIR Data Structures (PROPOSED)

```
HIR
  Module
    functions: Vec of Function
    types: Vec of TypeDecl
    constants: Vec of Constant
  Function
    name: String
    domain: DomainId (verified in domain analysis)
    params: Vec of (String, Type)
    return_type: Type
    locals: Vec of (LocalId, Type)
    body: Vec of Statement
    source_location: SourceLoc
  Statement
    Assign { target: LocalId, value: Expr }
    Return(Expr)
    If { condition: Expr, then: Vec of Statement, else: Vec of Statement }
    While { condition: Expr, body: Vec of Statement }
    Block(Vec of Statement)
    ExprStmt(Expr)
  Expr
    Local(LocalId)
    Literal(LiteralValue)
    BinaryOp { op: BinOp, left: Expr, right: Expr }
    UnaryOp { op: UnaryOp, operand: Expr }
    Call { function: FunctionId, args: Vec of Expr, is_cross_domain: bool }
    Index { target: Expr, index: Expr }
    Field { target: Expr, field: FieldId }
    Construct { type_id: TypeId, fields: Vec of (FieldId, Expr) }
    Cast { value: Expr, target_type: Type }
  Type
    Primitive(PrimitiveType)
    Named(TypeId)
    Function { params: Vec of Type, return_type: Box of Type }
    Reference { mutable: bool, inner: Box of Type }
```

## Bytecode Format (PROPOSED)

Stack-based bytecode for the VM:

```
Functions
  Each function contains:
    - Name
    - Parameter types
    - Return type
    - Local variable types
    - Bytecode instructions

Instructions
  Push(value)           Push a constant
  Pop                   Discard top of stack
  LocalGet(id)          Push a local variable
  LocalSet(id)          Pop into a local variable
  Call(fn_id, argc)     Call a function
  Return                Return from function
  Jump(target)          Unconditional jump
  ConditionalJump(t)    Pop and jump if false
  Add, Sub, Mul, Div    Arithmetic
  Equal, Less, Greater  Comparison
  Construct(type_id)    Construct a struct
  FieldGet(field_id)    Get a struct field
  FieldSet(field_id)    Set a struct field
```

### Why Stack-Based

Stack-based bytecode is:

- Simple to implement
- Simple to generate from HIR
- Easy to interpret
- Compact (no register allocation needed)

Register-based bytecode is more efficient but harder to generate and implement. For the MVP, stack-based is correct.

## Bytecode Verification (LOCKED)

The VM verifies bytecode before execution:

- Type checking (every stack operation has the correct type)
- Bounds checking (no out-of-bounds jumps)
- Stack depth verification (no stack overflow/underflow)
- Function call arity checking

This catches compiler bugs and malformed bytecode. It is a safety net, not a performance optimization.
