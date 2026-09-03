# AXIOM Type System (Revised)

## Semantic Requirements (LOCKED)

1. All types are checked at compile time.
2. The compiler infers types where possible; explicit annotations are required only when necessary for disambiguation.
3. Types are identified by name (nominal), not by structure.
4. The language supports sum types and product types.
5. The absence of a value is represented in the type system (not by a null sentinel).
6. A function that can fail must declare that it can fail, and the failure must be handled or propagated by the caller.
7. The type system must be sound: a program that type-checks must not exhibit type errors at runtime.
8. The type system must track execution domain membership.

## What the Programmer Expresses (PROPOSED)

### Primitives

- Integer types (at least 32-bit and 64-bit, signed and unsigned)
- Floating-point types (at least 64-bit)
- Boolean
- Character
- String

### Compound Types

- Tuples (ordered, fixed-size, heterogeneous)
- Structs (named fields, named type)
- Enums (sum types with named variants, variants may carry data)
- Arrays (fixed-size, homogeneous)
- Lists (dynamic-size, homogeneous)

### Function Types

- Functions have declared parameter types and return types
- Higher-order functions (functions as values) are supported

### Absence (LOCKED requirement)

The type system must distinguish between a value being present and a value being absent. This is represented as a sum type with two variants. The exact type name, variant names, and syntax are PROPOSED. The semantic requirement is LOCKED.

### Failure (LOCKED requirement)

A function that can fail must declare the failure in its return type. This is represented as a sum type with two variants. The caller must handle or propagate the failure. The exact type name, variant names, propagation syntax, and conversion mechanism are PROPOSED. The semantic requirement is LOCKED.

### Domain Membership (LOCKED requirement)

Each type belongs to an execution domain. Types defined in the server domain are server types. Types defined in the client domain are client types. When a type crosses a domain boundary, it must satisfy the transmissibility constraint. The transmissibility constraint is an OPEN research question.

## Why No Exceptions

Exceptions violate AXIOM's explicit-over-implicit principle: invisible control flow, type erasure, resource leaks, and the compiler cannot verify that errors are handled. A failure type in the return signature satisfies all locked requirements.

## Why Nominal Typing

Structural typing creates implicit coupling. Changing a field name silently breaks code that uses the type structurally. Nominal typing means a type satisfies an interface only if it explicitly declares it. More verbose but more predictable and refactoring-safe.

## Generics (PROPOSED, deferred from MVP)

Parametric polymorphism with type constraints. The syntax and constraint mechanism are PROPOSED. Deferred from MVP.

## Traits / Interfaces (OPEN)

The mechanism for defining shared behavior across types is an OPEN research question. Options include Rust-style traits, Go-style interfaces, Swift-style protocols, or a new design.

## Effect Tracking (OPEN, future)

Functions may declare what effects they have (I/O, mutation, throwing). This is an OPEN research question for future versions.

## Type System Soundness (LOCKED)

The type system is designed to be sound: no null pointer dereferences, no use-after-free, no data races, no type confusion, no unhandled failures. Soundness is the hard constraint.
