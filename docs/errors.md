# AXIOM Error Handling (Revised)

## Semantic Requirements (LOCKED)

1. A function that can fail must declare that it can fail in its return type.
2. The caller must handle or propagate the failure. The compiler must verify this.
3. Failures must be values (not exceptions, not hidden control flow).
4. The type system must track the failure type so the caller knows what failures to expect.
5. The programmer must be able to define custom failure types.
6. For truly unrecoverable conditions (programmer errors, assertion failures), a termination mechanism must exist that is distinct from recoverable failure.

## What the Programmer Expresses (PROPOSED)

### Recoverable Failure (LOCKED requirement)

A function that can fail returns a type that carries either the success value or the failure value. The exact type name, variant names, and syntax are PROPOSED. The semantic requirements are LOCKED.

The programmer must handle or propagate the failure. The compiler enforces this. Unhandled failures are a compile error.

**Propagation (LOCKED requirement, PROPOSED syntax):** The programmer must be able to concisely propagate a failure from the current function to the caller. The exact operator or syntax is PROPOSED. The semantic requirement (concise propagation is possible) is LOCKED.

**Failure conversion (LOCKED requirement, PROPOSED syntax):** When propagating a failure, the programmer must be able to convert between failure types. The exact mechanism is PROPOSED. The semantic requirement (conversion is possible) is LOCKED.

### Unrecoverable Failure (LOCKED requirement)

For conditions that indicate programmer bugs (assertion failures, unreachable code), the language provides a termination mechanism. This is distinct from recoverable failure: it is not returned as a value, it cannot be caught by the failure-handling mechanism, and it terminates the current execution unit.

The exact syntax is PROPOSED. The semantic requirements (distinct from recoverable failure, terminates execution, cannot be caught) are LOCKED.

## Why No Exceptions

Exceptions violate AXIOM's explicit-over-implicit principle:

1. **Invisible control flow** - A function call can throw, but the source code does not show it. The programmer must read documentation or source code to know which functions throw.
2. **Type erasure** - Exception types are erased at the catch boundary. The compiler cannot verify that all exceptions are caught.
3. **Resource leaks** - Exceptions can bypass cleanup code. RAII-style cleanup requires special handling.
4. **The compiler cannot verify handling** - A function that throws can be called without any indication that the caller handles the failure.
5. **Performance cost** - Exception handling is expensive on some platforms and prevents certain optimizations.

A failure type in the return signature satisfies all locked requirements: the failure is visible in the type, the compiler can verify handling, cleanup code executes normally, and there is no hidden control flow.

## Comparison with Existing Approaches

| Approach | Pros | Cons | AXIOM's choice |
|----------|------|------|----------------|
| Exceptions (Java, Python) | Concise syntax, no return type pollution | Invisible control flow, type erasure, resource leaks | Rejected |
| Result types (Rust) | Explicit, composable, type-safe | Verbose without propagation operator | Selected (semantic model) |
| Error codes (C) | Simple, no overhead | Manual checking, easy to forget, no type safety | Rejected |
| Try/catch (Go) | Simple error handling | Multiple return values, easy to ignore errors | Partially considered |
| Effect systems | Type-safe, composable | Complex to implement, complex for programmers | Deferred (future research) |

## Error Handling Patterns (PROPOSED syntax)

The following patterns demonstrate the semantic requirements. The exact syntax is PROPOSED.

### Direct handling

The programmer handles the failure explicitly by inspecting the result.

### Propagation

The programmer propagates the failure to the caller using the propagation mechanism.

### Conversion

The programmer converts between failure types when propagating across module boundaries.

### Accumulation

The programmer collects multiple failures from a batch operation.

## Compiler Error Reporting (LOCKED)

Compiler errors for incorrect error handling must be clear and specific:

- Include source location
- Explain what was expected
- Suggest possible fixes
- Show the type that was returned vs what was expected

## Error Context (OPEN)

Attaching source location and backtrace information to failures at runtime. This is useful for debugging but is an OPEN research question: the mechanism needs to be designed without adding runtime overhead to the success path.
