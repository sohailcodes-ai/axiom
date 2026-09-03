# AXIOM Language Core Specification — Phase 1

This document specifies the core semantics of the AXIOM programming language. It covers the minimum set of language features required for a useful, compilable, and runnable language.

Every decision is classified:

- **LOCKED** — Semantic requirement. Will not change without documented justification.
- **PROPOSED** — Best current design. May change based on evidence.
- **OPEN** — Research question. Requires further investigation.

Syntax is used only to illustrate semantic concepts. No syntax in this document is final unless explicitly marked LOCKED.

---

## 1. Values and Types

### 1.1 Semantic Model

Every AXIOM expression produces a value. Every value has a type. A type describes the set of possible values and the operations that can be performed on them.

Types are properties of values at compile time. They do not exist at runtime in the VM target (the VM uses tagged values or type tags, but the programmer does not see this). Types exist at runtime only insofar as they affect representation (size, alignment, dispatch).

### 1.2 Primitive Types

**LOCKED:**

- The language must have at least one integer type, at least one floating-point type, a boolean type, and a string type.
- Integer types must have defined width. The programmer must be able to choose a width appropriate to the domain.
- Floating-point types must follow IEEE 754 semantics.
- Boolean has exactly two values: true and false.

**PROPOSED:**

| Type | Width | Description |
|------|-------|-------------|
| i8, i16, i32, i64 | 8, 16, 32, 64 bits | Signed integers |
| u8, u16, u32, u64 | 8, 16, 32, 64 bits | Unsigned integers |
| f32, f64 | 32, 64 bits | IEEE 754 floats |
| bool | 1 bit (logical) | Boolean |
| char | 32 bits | Unicode scalar value |

**OPEN:**

- Should there be a default integer type (e.g., `int` that is platform-sized)?
- Should there be a `bigint` / arbitrary-precision integer type?
- Should `char` exist or should characters always be part of strings?

### 1.3 String Type

**LOCKED:**

- Strings are immutable. Once created, the contents of a string cannot be changed.
- Strings are a sequence of bytes encoding Unicode text.
- String operations (concatenation, slicing) produce new strings, never mutate in place.

**PROPOSED:**

- Encoding: UTF-8 (leading candidate)
- Strings are heap-allocated values (or stack-allocated for short strings, at the backend's discretion)
- String literals are compile-time constants

**OPEN:**

- UTF-8 vs UTF-16 vs rope structure
- Should string slicing be O(1) (requires pre-computed offsets) or O(n) (requires scanning)?
- Should there be a mutable string builder type for performance-critical construction?

### 1.4 Compound Types: Tuples

**LOCKED:**

- A tuple is an ordered, fixed-size, heterogeneous collection of values.
- Each element in a tuple has a position (index) and a type.
- Tuples are value types: assigning a tuple copies all elements (or moves them, per ownership rules).
- Tuples of the same shape (same types in the same order) are the same type.

**PROPOSED:**

- Tuple syntax: `(Int, String, Bool)` for the type, `(1, "hello", true)` for the value
- Tuples can be destructured: `let (a, b, c) = my_tuple;`
- Tuples with one element are not distinct from their inner type (no `(T)` syntax, only `(T,)` if needed for disambiguation)

**OPEN:**

- Maximum tuple size
- Named tuples vs anonymous structs
- Should 0-element tuples () (unit type) exist?

### 1.5 Compound Types: Structs

**LOCKED:**

- A struct is a named product type with named fields.
- Each field has a name and a type.
- Fields are accessed by name, not by position.
- Structs are value types: assigning a struct copies all fields (or moves them, per ownership rules).
- Two struct types with different names are distinct types, even if they have identical field definitions (nominal typing).

**PROPOSED:**

- Struct syntax: `struct User { name: String, age: Int }`
- Field access: `user.name`
- Struct construction: `User { name: "Alice", age: 30 }`
- Structs can have methods (functions associated with the type)

**OPEN:**

- Should structs support default field values?
- Should struct fields be immutable by default or mutable by default?
- Should there be struct update syntax (`User { ..existing, age: 31 }`)?

### 1.6 Compound Types: Enums (Sum Types)

**LOCKED:**

- An enum is a named sum type with named variants.
- A value of an enum type is exactly one of its variants at any time.
- Variants may carry data (each variant has a payload type).
- Variants without data are simple markers (unit variants).
- Enums are value types.

**PROPOSED:**

- Enum syntax: `enum Result { Ok(T), Err(E) }` (generic syntax deferred)
- Variant syntax: `Result.Ok(42)`, `Result.Err("failed")`
- Pattern matching on enums is the primary way to use them
- Enums can have methods

**OPEN:**

- Should variants be namespaced under the enum name (`Result.Ok`) or in the global namespace (`Ok`)?
- Should there be exhaustive matching enforcement? (Likely yes, but the mechanism needs design.)
- Should enums support methods defined directly on variants?

### 1.7 Compound Types: Arrays

**LOCKED:**

- An array is a fixed-size, homogeneous, contiguous collection of values.
- The size is part of the type: `[Int; 5]` is a different type from `[Int; 10]`.
- Array elements are accessed by index (0-based).
- Out-of-bounds access is a runtime error (not undefined behavior).

**PROPOSED:**

- Array syntax: `[1, 2, 3]` for values, `[Int; 3]` for the type
- Index access: `arr[0]`
- Arrays are stack-allocated (size known at compile time)
- Arrays support iteration

**OPEN:**

- Should out-of-bounds access panic (terminate) or return a result type?
- Should there be a bounds-checking mode (checked in debug, unchecked in release)?

### 1.8 Compound Types: Lists (Dynamic Arrays)

**LOCKED:**

- A list is a dynamic-size, homogeneous collection of values.
- Lists can grow and shrink.
- Lists are heap-allocated.
- Out-of-bounds access is a runtime error.

**PROPOSED:**

- List type: `List<Int>` (generic syntax deferred, concrete type for MVP)
- Construction: `List.new()` or `[1, 2, 3]` (inferred as list)
- Operations: `push`, `pop`, `len`, `get`, `set`
- Lists are reference types (heap-allocated, shared via ownership/borrowing)

**OPEN:**

- Should lists be resizable arrays (like Rust Vec) or linked lists?
- Should there be a fixed-capacity list type?
- Should list operations return results for fallible operations?

### 1.9 Function Types

**LOCKED:**

- Functions are first-class values. They can be passed as arguments, returned from other functions, and stored in variables.
- Function types describe the parameter types and return type.
- Functions have exactly one return value (tuples for multiple values).

**PROPOSED:**

- Function type: `fn(Int, String) -> Bool`
- Function declaration: `fn add(a: Int, b: Int) -> Int { return a + b; }`
- Anonymous functions (lambdas): `fn(x) x + 1`
- Higher-order functions: functions that take or return functions

**OPEN:**

- Should there be a distinction between named functions and anonymous functions?
- Should closures capture by value or by reference?
- Should there be a shorthand for single-expression functions?

### 1.10 Reference Types

**LOCKED:**

- The programmer must be able to express borrowing (reading a value without taking ownership).
- References come in two forms: shared (read-only) and exclusive (read-write).
- A shared reference allows reading but not writing.
- An exclusive reference allows reading and writing, but only one exclusive reference can exist at a time.
- References must not outlive the value they reference.

**PROPOSED:**

- Shared reference type: `&T`
- Exclusive reference type: `&mut T`
- Reference creation: `&x` for shared, `&mut x` for exclusive
- Reference access: automatic dereferencing (no explicit dereference operator needed in most cases)

**OPEN:**

- Should references be nullable? (Likely no, given absence is a sum type.)
- Should raw pointers exist for unsafe code?
- How should reference lifetimes be expressed in function signatures?

### 1.11 Unit Type

**LOCKED:**

- There must be a type that represents "no meaningful value." This is used for functions that perform an action without producing a result.

**PROPOSED:**

- Unit type: `()` (zero-element tuple)
- Unit value: `()`
- Functions that return nothing have return type `()` (often elided in syntax)

**OPEN:**

- Should `()` be called "unit" or something else?
- Should the return type be elided when it is `()`?

---

## 2. Bindings and Mutation

### 2.1 Semantic Model

A binding associates a name with a value. Bindings are the primary mechanism for giving names to computed values.

### 2.2 Immutable Bindings

**LOCKED:**

- The default binding is immutable. Once a name is bound to a value, the name cannot be rebound to a different value.
- Immutable bindings must be initialized at the time of creation.

**PROPOSED:**

- Binding syntax: `let x = 42;`
- The type can be annotated: `let x: Int = 42;`
- The type can be inferred: `let x = 42;` (inferred as Int)

### 2.3 Mutable Bindings

**LOCKED:**

- The programmer must be able to create a mutable binding that can be reassigned.
- Mutability must be explicit (not the default).

**PROPOSED:**

- Mutable binding syntax: `let mut x = 42;`
- Reassignment: `x = 43;`
- Mutability is a property of the binding, not the value

### 2.4 Shadowing

**LOCKED (requirement to decide):**

- The language must decide whether shadowing is allowed (a new binding with the same name as an existing binding).

**PROPOSED:**

- Shadowing is allowed. A new `let` binding with the same name as an existing binding creates a new binding that shadows the old one.
- Shadowing does not mutate the old binding. The old value continues to exist until it goes out of scope.
- Shadowing with `let mut` is also allowed.

**OPEN:**

- Should shadowing be allowed within the same scope?
- Should there be a warning for shadowing?
- Should shadowing be disallowed for mutable bindings?

### 2.5 Destruction on Reassignment

**LOCKED:**

- When a mutable binding is reassigned, the old value is dropped (its cleanup runs). This is target-independent: the VM's GC handles it, the native target's RC handles it, etc.

**PROPOSED:**

- Reassignment of a mutable binding drops the old value
- The programmer can observe this through custom cleanup behavior
- The compiler may optimize away the drop if the value is provably unused

**OPEN:**

- Should destruction on reassignment be guaranteed or best-effort?
- Should there be a way to transfer a value out of a mutable binding without dropping it?

---

## 3. Functions

### 3.1 Semantic Model

A function is a named, parameterized computation that produces a return value. Functions are the primary unit of code organization.

### 3.2 Function Declaration

**LOCKED:**

- Functions must have declared parameter types and a declared return type.
- The compiler must verify that the function body is consistent with its signature.
- Functions must return a value of the declared type, or propagate a failure if the return type is a failure type.

**PROPOSED:**

- Declaration syntax: `fn name(param1: Type1, param2: Type2) -> ReturnType { body }`
- The return type can be elided when the function returns `()`
- Parameters are immutable bindings by default; `mut` can be added for mutable parameters

### 3.3 Function Calls

**LOCKED:**

- Arguments are passed to functions. The number and types of arguments must match the function signature.
- The result of a function call is the return value.
- If the return type is a failure type, the caller must handle or propagate the failure.

**PROPOSED:**

- Call syntax: `name(arg1, arg2)`
- Named arguments: `name(param1: arg1, param2: arg2)` (PROPOSED, for readability)
- The compiler verifies argument count and types at compile time

### 3.4 First-Class Functions

**LOCKED:**

- Functions are first-class values. They can be stored in variables, passed as arguments, and returned from other functions.

**PROPOSED:**

- A function name refers to the function value
- Anonymous functions: `fn(x: Int) -> Int { return x + 1; }`
- Shorthand anonymous functions (for single expressions): `fn(x) x + 1`

### 3.5 Closures

**LOCKED:**

- Functions must be able to capture values from their enclosing scope.
- Captured values follow the same ownership and borrowing rules as other values.

**PROPOSED:**

- Closures capture by value (the closure owns a copy of the captured value)
- Closures can capture by reference if borrowing is used
- The capture mode is determined by how the closure uses the captured value (read-only = shared borrow, mutable = exclusive borrow, consumed = ownership transfer)

**OPEN:**

- Should closures capture by value by default or by reference by default?
- Should there be explicit capture syntax?
- How should closure lifetimes interact with reference lifetimes?

### 3.6 Recursion

**LOCKED:**

- Functions may call themselves directly or indirectly (recursion).
- The compiler must detect and reject infinite recursion at compile time when possible (tail-call optimization is not required but is permitted).

**PROPOSED:**

- Recursive functions work as in other languages
- No special syntax required
- The compiler may warn about likely infinite recursion

### 3.7 Higher-Order Functions

**LOCKED:**

- Functions must be able to accept other functions as parameters and return functions as results.

**PROPOSED:**

- Function types as parameters: `fn map(items: List<Int>, f: fn(Int) -> Int) -> List<Int>`
- Function types as return values: `fn compose(f: fn(Int) -> Int, g: fn(Int) -> Int) -> fn(Int) -> Int`

---

## 4. Scopes

### 4.1 Semantic Model

A scope is a region of source code where names are visible. Names are introduced into a scope and are no longer visible when the scope ends.

### 4.2 Block Scope

**LOCKED:**

- A block `{ ... }` creates a new scope.
- Names introduced in a block are visible within that block and any nested blocks.
- Names introduced in a block are not visible outside that block.
- When a block ends, all names introduced in that block go out of scope. Values that go out of scope are dropped (cleanup runs).

**PROPOSED:**

- Blocks are delimited by `{` and `}`
- A block can contain statements and optionally a final expression (the block's value)
- The value of a block is the value of the final expression, or `()` if there is no final expression

### 4.3 Function Scope

**LOCKED:**

- A function body is a block scope.
- Parameters are names in the function's scope.
- The function's return value is the value of the final expression in the function body.

### 4.4 Module Scope

**LOCKED:**

- Each module has its own scope.
- Names declared at the top level of a module are visible to other modules through the module system.
- Names declared inside functions are local to that function.

**PROPOSED:**

- Modules correspond to files (one file = one module)
- Top-level declarations are public by default (or private by default — OPEN question)

### 4.5 Lifetime of Bindings

**LOCKED:**

- A binding's lifetime is the scope in which it is visible.
- When a binding goes out of scope, the value it holds is dropped.
- Dropping a value runs its cleanup behavior (if any) and then reclaims the memory.

**PROPOSED:**

- The order of dropping is reverse declaration order within a scope
- The compiler may reorder drops if it can prove the reordering does not affect observable behavior

---

## 5. Expressions

### 5.1 Semantic Model

An expression is a computation that produces a value. Every expression has a type. Expressions can be composed: the result of one expression can be used as an operand in another expression.

### 5.2 Literal Expressions

**LOCKED:**

- Integer literals: `42`, `0xff`, `0b1010`, `0o77`
- Float literals: `3.14`, `1.0e10`
- Boolean literals: `true`, `false`
- String literals: `"hello world"`
- Character literals: `'a'`
- Unit literal: `()`

**PROPOSED:**

- Integer literals can use `_` as a separator: `1_000_000`
- String literals support escape sequences: `\n`, `\t`, `\\`, `\"`
- Multi-line strings: delimited by triple quotes or similar (OPEN)

### 5.3 Arithmetic Expressions

**LOCKED:**

- The language must support addition, subtraction, multiplication, and division for numeric types.
- Division by zero is a runtime error.
- Integer overflow behavior must be defined (panic in debug, wrap in release, or always panic — OPEN).

**PROPOSED:**

- Operators: `+`, `-`, `*`, `/`
- Modulo: `%`
- Negation: `-x`
- No implicit type coercion between integers and floats (explicit conversion required)
- No implicit widening between integer types (explicit conversion required)

**OPEN:**

- Integer overflow: panic, wrap, or saturate?
- Should there be checked arithmetic operations that return results?
- Should there be a `**` (power) operator?

### 5.4 Comparison Expressions

**LOCKED:**

- The language must support equality and ordering comparisons for comparable types.
- Comparison operators return boolean values.
- Only values of the same type can be compared (no implicit coercion).

**PROPOSED:**

- Equality: `==`, `!=`
- Ordering: `<`, `>`, `<=`, `>=`
- Comparison is defined for integers, floats, chars, strings, and tuples (element-wise)
- Structs and enums are not comparable by default (must implement comparison explicitly)

### 5.5 Logical Expressions

**LOCKED:**

- The language must support logical AND, OR, and NOT for boolean values.
- Logical AND and OR are short-circuit: the right operand is not evaluated if the left operand determines the result.

**PROPOSED:**

- AND: `&&`
- OR: `||`
- NOT: `!`

### 5.6 Assignment Expressions

**LOCKED:**

- Assignment changes the value of a mutable binding.
- Assignment is not an expression (it does not produce a value). This prevents `if (x = 5)` bugs.

**PROPOSED:**

- Assignment syntax: `x = value;`
- Compound assignment: `x += value;`, `x -= value;`, etc.
- Assignment is a statement, not an expression

### 5.7 Function Call Expressions

See Section 3.3.

### 5.8 Index Expressions

**LOCKED:**

- The language must support indexing into arrays and lists by integer position.
- Out-of-bounds access is a runtime error.

**PROPOSED:**

- Index syntax: `arr[0]`
- Index returns a reference to the element (for borrowing semantics)
- Out-of-bounds access panics (terminates the current task)

### 5.9 Field Access Expressions

**LOCKED:**

- The language must support accessing struct fields by name.

**PROPOSED:**

- Field access syntax: `user.name`
- Field access returns a reference to the field (for borrowing semantics)
- Field access on non-struct types is a compile error

### 5.10 Tuple Index Expressions

**PROPOSED:**

- Tuple elements can be accessed by position: `tup.0`, `tup.1`
- Or by destructuring: `let (a, b) = tup;`

### 5.11 Block Expressions

**LOCKED:**

- A block `{ ... }` is an expression. Its value is the value of the final expression in the block.
- Blocks can contain statements followed by a final expression.

**PROPOSED:**

- Block syntax: `{ stmt1; stmt2; expr }`
- The final expression (without a trailing semicolon) is the block's value
- If the block ends with a semicolon, the value is `()`

### 5.12 If Expressions

See Section 6.1.

### 5.13 Match Expressions

See Section 6.3.

### 5.14 Type Annotation Expressions

**LOCKED:**

- The programmer must be able to explicitly annotate the type of an expression.
- Type annotations do not change runtime behavior; they are compile-time checks.

**PROPOSED:**

- Annotation syntax: `expr: Type`
- Used in bindings: `let x: Int = 42;`
- Used in function parameters: `fn add(a: Int, b: Int) -> Int`
- Used for disambiguation when inference is insufficient

### 5.15 Operator Precedence (PROPOSED)

The following precedence is proposed (highest to lowest):

1. Function call, field access, index
2. Unary `-`, `!`
3. `*`, `/`, `%`
4. `+`, `-`
5. `<`, `>`, `<=`, `>=`
6. `==`, `!=`
7. `&&`
8. `||`
9. Assignment (`=`, `+=`, etc.)

All binary operators are left-associative.

---

## 6. Control Flow

### 6.1 If Expressions

**LOCKED:**

- The language must have conditional execution based on a boolean condition.
- If-expressions produce a value (they are expressions, not just statements).
- Both branches must have the same type (or the type must be inferrable).

**PROPOSED:**

- If syntax: `if condition { then_branch } else { else_branch }`
- The `else` branch is required if the if-expression produces a value
- If without else produces `()`
- Chaining: `if ... else if ... else ...`

### 6.2 While Expressions

**LOCKED:**

- The language must have a looping construct that repeats while a condition is true.
- While-expressions produce a value (the value of the last expression in the loop body, or `()`).

**PROPOSED:**

- While syntax: `while condition { body }`
- The loop runs as long as the condition evaluates to `true`
- `break` exits the loop early
- `continue` skips to the next iteration

### 6.3 Match Expressions (Pattern Matching)

**LOCKED:**

- The language must have pattern matching on values.
- Pattern matching must be exhaustive: the compiler must verify that all possible cases are handled.
- Patterns can destructure values (tuples, structs, enums).

**PROPOSED:**

- Match syntax:
```
match value {
    pattern1 => result1,
    pattern2 => result2,
    _ => default_result,
}
```
- Patterns: literals, identifiers (bindings), wildcards (`_`), tuple destructuring, struct destructuring, enum variant destructuring
- Match is an expression: each arm produces a value
- The compiler verifies exhaustiveness at compile time

**OPEN:**

- Should match be exhaustive by default with an opt-in wildcard?
- Should there be a guard syntax (`pattern if condition => result`)?
- Should match support or-patterns (`pattern1 | pattern2 => result`)?

### 6.4 For Expressions

**LOCKED (requirement to decide):**

- The language must have a way to iterate over collections.

**PROPOSED:**

- For syntax: `for item in collection { body }`
- The `in` expression produces an iterator (conceptually)
- `for` is syntactic sugar for while-loop with iterator protocol
- `break` and `continue` work inside `for` loops

**OPEN:**

- Should there be an explicit iterator protocol?
- Should `for` support index-based iteration (`for i in 0..10`)?
- Should `for` support destructuring (`for (key, value) in map`)?

### 6.5 Break and Continue

**LOCKED:**

- `break` must exit the innermost loop.
- `continue` must skip to the next iteration of the innermost loop.

**PROPOSED:**

- `break` without a value produces `()`
- `break value` produces a value from the loop (loops as expressions)
- `continue` does not produce a value

### 6.6 Return

**LOCKED:**

- A function must be able to return a value early (before the final expression).
- The return value must match the function's declared return type.

**PROPOSED:**

- Return syntax: `return expr;`
- `return` without an expression returns `()`
- `return` is a statement, not an expression

### 6.7 Tail Calls (OPEN)

- Should the language guarantee tail-call optimization?
- Should there be a tail-call annotation?
- This is an OPEN research question.

---

## 7. Errors and Absence

### 7.1 Error Handling Model

**LOCKED:**

- A function that can fail must declare the failure in its return type.
- The caller must handle or propagate the failure. The compiler must verify this.
- Failures are values, not exceptions. There is no hidden control flow.
- The type system tracks the failure type so the caller knows what failures to expect.

### 7.2 Failure Type

**LOCKED (semantic requirement):**

- There must be a sum type with two variants: one carrying the success value, one carrying the failure value.
- Functions that can fail return this type.
- The caller must handle both variants (success and failure).

**PROPOSED (syntax and naming):**

- The exact type name is OPEN (candidates: `Result`, `Outcome`, `Try`, or a new name)
- The success variant name is OPEN (candidates: `Ok`, `Success`, `Value`, or a new name)
- The failure variant name is OPEN (candidates: `Err`, `Failure`, `Error`, or a new name)

### 7.3 Failure Propagation

**LOCKED (semantic requirement):**

- The programmer must be able to concisely propagate a failure from the current function to the caller.
- Propagation must be explicit (not automatic).
- The compiler must verify that propagated failures are compatible with the caller's expected failure type.

**PROPOSED (syntax):**

- A propagation operator (candidates: `?`, `!`, `try`, or a keyword) that:
  1. Evaluates the expression
  2. If the result is the success variant, unwraps and continues
  3. If the result is the failure variant, returns early from the current function with that failure

### 7.4 Failure Conversion

**LOCKED (semantic requirement):**

- When propagating a failure, the programmer must be able to convert between failure types.
- Conversion must be explicit or through a defined conversion path.

**PROPOSED:**

- Automatic conversion through an `Into`-like mechanism (the failure type implements conversion to the target failure type)
- Manual conversion through mapping functions

### 7.5 Absence Type

**LOCKED (semantic requirement):**

- The type system must distinguish between a value being present and a value being absent.
- Absence is not represented by a null sentinel. It is a sum type.
- The compiler must enforce that absent values are handled.

**PROPOSED (syntax):**

- The exact type name is OPEN (candidates: `Option`, `Maybe`, `Present`, or a new name)
- The present variant is OPEN (candidates: `Some`, `Just`, `Value`, or a new name)
- The absent variant is OPEN (candidates: `None`, `Nothing`, `Absent`, or a new name)

### 7.6 Absence Handling

**LOCKED (semantic requirement):**

- The programmer must be able to provide a default value for absent values.
- The programmer must be able to transform present values while preserving absence.
- The compiler must enforce that absent values are handled before using the inner value.

**PROPOSED:**

- Methods on the absence type: `unwrap_or(default)`, `map(f)`, `and_then(f)`
- Pattern matching on the absence type
- The propagation operator (from 7.3) can also be used with absence types

### 7.7 Panic (Unrecoverable Failure)

**LOCKED (semantic requirement):**

- For conditions that indicate programmer bugs (assertion failures, unreachable code), a termination mechanism must exist.
- Panic is distinct from recoverable failure: it is not returned as a value, it cannot be caught by the failure-handling mechanism, and it terminates the current execution unit.

**PROPOSED:**

- Panic syntax: `panic("message")` or similar
- Panic unwinds the stack and terminates the current task
- Panic is not for normal error handling; it is for programmer errors

### 7.8 Unreachable Code

**LOCKED:**

- The language must have a way to mark code as unreachable.
- If unreachable code is executed, it is equivalent to a panic.

**PROPOSED:**

- An `unreachable` expression that panics if executed
- The compiler uses unreachable annotations for optimization and exhaustiveness checking

---

## 8. Memory and Resource Semantics

### 8.1 Source-Level Semantics

These semantics are target-independent. The same AXIOM source code has the same meaning on VM, native, and WASM targets.

### 8.2 Ownership

**LOCKED:**

- Every value has exactly one owner at any time.
- When ownership transfers (value passed to a function, assigned to another binding), the original binding can no longer use the value.
- When the owner goes out of scope, the value is dropped (cleanup runs).

**PROPOSED:**

- Ownership transfer is the default when passing values to functions or assigning them
- The programmer can borrow instead of transferring ownership (see 8.3)
- Ownership transfer is implicit (no explicit syntax required for the common case)

### 8.3 Borrowing

**LOCKED:**

- The programmer must be able to read a value without taking ownership (shared borrow).
- The programmer must be able to read and write a value without taking ownership (exclusive borrow).
- At any given time, either one exclusive borrow or any number of shared borrows can exist for a value.
- Borrows must not outlive the value they reference.

**PROPOSED:**

- Shared borrow: create a reference that can read the value
- Exclusive borrow: create a reference that can read and write the value
- The borrow checker enforces the borrowing rules at compile time (when implemented)
- In the MVP, the GC ensures safety; borrow checking is deferred

### 8.4 Cleanup

**LOCKED:**

- Values are cleaned up when they go out of scope.
- Cleanup runs in reverse declaration order within a scope.
- Types can define custom cleanup behavior.

**PROPOSED:**

- Custom cleanup: a special method or function that runs when the value is dropped
- The cleanup function can access the value's fields
- The cleanup function cannot prevent the drop (the value is always cleaned up)

### 8.5 Dynamic Allocation

**LOCKED:**

- The language must provide a mechanism for values that outlive any single owner.
- Dynamic allocation must be managed (not raw malloc/free).

**PROPOSED:**

- Heap allocation is implicit (the compiler decides when to allocate on the heap)
- The GC (VM target) or reference counting (native target) manages heap memory
- The programmer does not manually allocate or deallocate heap memory in normal code

### 8.6 Target-Independent Semantics

The following behaviors are guaranteed to be the same across all targets:

1. Ownership transfer semantics (value moves, old binding invalidated)
2. Borrowing rules (shared vs exclusive, mutual exclusion)
3. Cleanup behavior (runs at scope exit, reverse order)
4. Drop semantics (custom cleanup runs, then memory reclaimed)
5. No use-after-free (guaranteed by the type system or runtime)
6. No data races (guaranteed by the borrow checker or runtime)

The following may differ between targets:

1. Performance characteristics (GC pauses vs RC overhead vs bump allocation)
2. Memory layout (stack vs heap decisions)
3. Actual memory reclamation mechanism (tracing GC, reference counting, region deallocation)

---

## 9. Concurrency Semantics

### 9.1 Semantic Model

AXIOM supports concurrent execution through lightweight tasks and typed message channels. Concurrency is a language-level concern, not an afterthought.

### 9.2 Task Spawning

**LOCKED:**

- The programmer must be able to express that a computation runs concurrently.
- A spawned task produces a handle that can be used to receive the result.
- Tasks are lightweight (not OS threads).

**PROPOSED:**

- A spawn primitive creates a task and returns a handle
- The handle can be awaited to receive the result
- The task runs concurrently with the spawning context

### 9.3 Structured Concurrency

**LOCKED:**

- Tasks are structured: they have a lifetime bounded by their spawning scope.
- When the spawning scope exits, all child tasks must either complete or be cancelled.
- The programmer does not need to manually track task lifetimes.

**PROPOSED:**

- When a scope exits, child tasks are cancelled (if still running)
- Awaiting a task before scope exit blocks until the task completes
- Cancellation is cooperative (the task checks for cancellation at defined points)

### 9.4 Channels

**LOCKED:**

- Tasks must communicate through typed channels (not shared memory by default).
- Channels carry values of a specific type.
- Sending and receiving are explicit operations.

**PROPOSED:**

- Channel creation produces a sender-receiver pair
- The sender can send values of the channel's type
- The receiver can receive values of the channel's type
- Sending and receiving are async (do not block the calling task)

### 9.5 Task Result Handling

**LOCKED:**

- The programmer must be able to wait for a task to produce a result.
- Waiting must not block other tasks (cooperative scheduling).

**PROPOSED:**

- Awaiting a task handle blocks the current task until the result is available
- If the task fails (panics), the await propagates the failure
- Multiple tasks can be awaited concurrently

### 9.6 Cancellation

**LOCKED:**

- Tasks must be cancellable when their result is no longer needed.
- Cancellation must be cooperative (not forced termination).

**PROPOSED:**

- When a scope exits, child tasks are cancelled
- A task checks for cancellation at await points
- When cancelled, a task runs its cleanup code and terminates
- A cancelled task does not produce a result

### 9.7 Data Race Prevention

**LOCKED:**

- The type system must prevent data races at compile time when feasible.
- Shared mutable state between tasks requires explicit synchronization.

**PROPOSED:**

- Channels are the primary mechanism for inter-task communication
- Shared state (when necessary) requires atomic types or locks
- The borrow checker prevents shared mutable references across tasks (when implemented)

### 9.8 Target-Specific Concurrency

The concurrency semantics are the same across targets. The runtime maps tasks differently:

| Target | Task Implementation | Scheduling |
|--------|-------------------|------------|
| VM | Green tasks | Cooperative scheduler |
| Native | OS threads + green tasks | Work-stealing thread pool |
| WASM | Web Workers or event loop | Browser event loop |

---

## 10. Modules

### 10.1 Semantic Model

A module is a unit of code organization. Modules provide namespace isolation and control over visibility.

### 10.2 Module Structure

**LOCKED:**

- A program is composed of one or more modules.
- Each module has its own scope.
- Modules can import names from other modules.

**PROPOSED:**

- A module corresponds to a file (one file = one module)
- The module name is derived from the file path
- A module can contain type declarations, function declarations, and constant declarations

### 10.3 Imports and Exports

**LOCKED:**

- Modules must be able to export names for use by other modules.
- Modules must be able to import names from other modules.
- Visibility must be controllable (not everything is public by default, or everything is — OPEN question).

**PROPOSED:**

- Import syntax: `import module_name` or `use module_name::item`
- Export: top-level declarations are exported by default (or require an explicit export keyword — OPEN)
- Nested imports: `import module::submodule::item`

### 10.4 Module Resolution

**LOCKED:**

- The compiler must be able to resolve module names to files.
- Module resolution must be deterministic.

**PROPOSED:**

- File-based resolution: `import foo` resolves to `foo.ax` in the same directory or a configured module path
- Package resolution: modules within a package are resolved relative to the package root
- Standard library modules are resolved from a known path

### 10.5 Circular Dependencies

**LOCKED:**

- The compiler must detect and reject circular dependencies at compile time.

**PROPOSED:**

- Circular imports are a compile error
- The compiler tracks the dependency graph and reports cycles

### 10.6 Module Initialization (OPEN)

- Should modules have initialization code that runs on first import?
- Should there be module-level constants that are computed at initialization?
- This is an OPEN research question.

---

## 11. Execution Domains

### 11.1 Semantic Model

An execution domain is a named compilation boundary. Code within a domain can only perform operations that the domain permits. The compiler verifies domain constraints at compile time.

### 11.2 Domain Declaration

**LOCKED:**

- The programmer must be able to declare which domain a function belongs to.
- The compiler must know the domain of every function.
- Domain declarations must be visible in the source code.

**PROPOSED (syntax candidates, not locked):**

- Block-level: `domain server { fn handle() { ... } }`
- Annotation: `@server fn handle() { ... }`
- Position: `fn handle() server -> Response { ... }`

### 11.3 Domain Capabilities

**LOCKED:**

- Each domain has a set of capabilities (operations that are permitted).
- The compiler verifies that a function only uses operations available in its domain.
- Operations not in the domain's capability set are compile errors.

**PROPOSED (capabilities per domain):**

| Domain | Permitted Operations | Forbidden Operations |
|--------|---------------------|---------------------|
| server | Filesystem, networking, databases, process spawning, full system access | DOM, browser APIs |
| client | DOM, fetch, localStorage, browser APIs | Filesystem, process spawning, raw memory access |
| worker | Computation, message passing, limited I/O | DOM, direct system calls |
| native | Raw memory, system calls, FFI, hardware access | Automatic memory safety |

### 11.4 Cross-Domain Calls

**LOCKED:**

- Cross-domain calls are not direct function calls.
- Cross-domain calls are message exchanges.
- The compiler generates serialization code for values that cross boundaries.
- Values crossing boundaries must be transmissible.

**PROPOSED:**

- A cross-domain call produces a request message and waits for a response message
- The compiler verifies that the request and response types are transmissible
- The runtime handles the actual message passing

### 11.5 Transmissibility

**LOCKED (semantic requirement):**

- When a value crosses a domain boundary, the compiler must verify the value is transmissible.
- Transmissibility is a type-system property.

**OPEN:**

- Which types are transmissible? (All types? Only certain types?)
- How is transmissibility defined? (Must implement a trait? Must be a certain kind of type?)
- What serialization format is used? (JSON, binary, custom protocol?)

### 11.6 Domain Analysis Stage

**LOCKED:**

- Domain analysis is a distinct compiler stage that runs after type checking.
- It verifies domain constraints, cross-domain call validity, and transmissibility.
- It produces a verified AST that downstream stages consume.

### 11.7 Single-Domain Programs (MVP)

**PROPOSED:**

- For the MVP, a program can declare a single domain (or no domain).
- Single-domain programs do not use cross-domain calls.
- The domain analysis stage is simplified for single-domain programs.
- Multi-domain programs are supported in Phase 4.

---

## 12. Cross-Cutting Concerns

### 12.1 Compile-Time Constants

**LOCKED:**

- The language must support compile-time constants.
- Compile-time constants are evaluated at compile time and substituted.

**PROPOSED:**

- Constant syntax: `const MAX_SIZE = 1024;`
- Constants must have a type annotation
- Constants can only depend on other constants (not runtime values)

### 12.2 Type Aliases

**LOCKED:**

- The programmer must be able to create alternative names for existing types.

**PROPOSED:**

- Alias syntax: `type UserId = Int;`
- Type aliases are transparent (the alias and the original type are interchangeable)
- Type aliases do not create new types (nominal typing is for structs and enums, not aliases)

### 12.3 Comments

**LOCKED:**

- The language must support comments that are ignored by the compiler.

**PROPOSED:**

- Single-line comments: `// comment`
- Multi-line comments: `/* comment */` (or nested multi-line — OPEN)
- Comments are preserved in the AST for documentation tooling

### 12.4 Error Recovery

**LOCKED:**

- The compiler must report all errors it finds in a compilation unit, not just the first error.
- Error recovery must be graceful: the compiler should continue parsing and type-checking after encountering errors.

**PROPOSED:**

- The compiler collects all errors and reports them together
- Each error includes a source location, description, and suggestion
- The compiler does not emit code if any errors were found

---

## Appendix: Decision Registry

### LOCKED Decisions (Phase 1)

| # | Section | Decision |
|---|---------|----------|
| L1 | 1.2 | Primitive types: defined-width integers, IEEE 754 floats, bool, char |
| L2 | 1.3 | Strings are immutable, UTF-8 (proposed encoding) |
| L3 | 1.4 | Tuples are ordered, fixed-size, heterogeneous, value types |
| L4 | 1.5 | Structs are nominal product types with named fields |
| L5 | 1.6 | Enums are nominal sum types with named variants |
| L6 | 1.7 | Arrays are fixed-size, homogeneous, 0-indexed, bounds-checked |
| L7 | 1.8 | Lists are dynamic-size, homogeneous, heap-allocated |
| L8 | 1.9 | Functions are first-class values |
| L9 | 1.10 | References: shared and exclusive borrowing |
| L10 | 2.2 | Default bindings are immutable |
| L11 | 2.3 | Mutability is explicit on bindings |
| L12 | 3.2 | Functions have declared parameter types and return types |
| L13 | 4.2 | Blocks create scopes |
| L14 | 4.5 | Values are dropped when bindings go out of scope |
| L15 | 5.1 | Every expression has a type |
| L16 | 5.6 | Assignment is a statement, not an expression |
| L17 | 6.1 | If-expressions produce values |
| L18 | 6.3 | Pattern matching must be exhaustive |
| L19 | 7.1 | Failures are values, not exceptions |
| L20 | 7.2 | Failure is a sum type in the return signature |
| L21 | 7.3 | Failure propagation is explicit |
| L22 | 7.5 | Absence is a sum type, not null |
| L23 | 7.7 | Panic is for unrecoverable errors, distinct from failure |
| L24 | 8.2 | Every value has one owner |
| L25 | 8.3 | Borrowing: shared and exclusive, mutual exclusion |
| L26 | 8.4 | Cleanup runs at scope exit |
| L27 | 9.2 | Tasks are lightweight, spawned with a concurrency primitive |
| L28 | 9.3 | Structured concurrency: tasks bounded by spawning scope |
| L29 | 9.4 | Communication through typed channels |
| L30 | 10.2 | Modules correspond to files |
| L31 | 10.5 | Circular dependencies are compile errors |
| L32 | 11.2 | Domain declarations are visible in source |
| L33 | 11.3 | Domains have defined capability sets |
| L34 | 11.4 | Cross-domain calls are message exchanges |
| L35 | 11.6 | Domain analysis is a distinct compiler stage |

### OPEN Research Questions (Phase 1)

| # | Section | Question |
|---|---------|----------|
| O1 | 1.2 | Default integer type? |
| O2 | 1.2 | Arbitrary-precision integer? |
| O3 | 1.3 | String encoding (UTF-8 vs UTF-16 vs rope)? |
| O4 | 1.3 | String slicing complexity? |
| O5 | 1.4 | Named tuples vs anonymous structs? |
| O6 | 1.5 | Enum variant namespacing? |
| O7 | 1.5 | Exhaustive matching mechanism? |
| O8 | 1.7 | Array out-of-bounds: panic or result? |
| O9 | 1.10 | Reference nullability? |
| O10 | 1.10 | Raw pointers for unsafe code? |
| O11 | 2.4 | Shadowing rules? |
| O12 | 4.4 | Module visibility (public by default or private)? |
| O13 | 5.3 | Integer overflow behavior? |
| O14 | 6.3 | Guard syntax in match? |
| O15 | 6.4 | Iterator protocol? |
| O16 | 6.7 | Tail-call optimization? |
| O17 | 7.2 | Failure type name and variant names? |
| O18 | 7.3 | Propagation operator syntax? |
| O19 | 7.5 | Absence type name and variant names? |
| O20 | 8.2 | Lifetime annotation syntax? |
| O21 | 9.3 | Cooperative cancellation mechanism? |
| O22 | 10.6 | Module initialization? |
| O23 | 11.5 | Transmissibility definition? |
| O24 | 11.5 | Cross-domain serialization format? |
