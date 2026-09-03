# AXIOM Memory Model (Revised)

## Semantic Requirements (LOCKED)

1. Resources must be cleaned up predictably. The programmer must be able to determine, from source code, when a resource is released.
2. The type system must prevent accidental sharing of mutable state between concurrent computations without explicit synchronization.
3. The compiler should detect use-after-free and data race conditions at compile time when feasible.
4. The programmer must be able to express ownership transfer (a value is no longer available at its old location after being used elsewhere).
5. The programmer must be able to express borrowing (reading a value without taking ownership).
6. The language must provide a mechanism for values that outlive any single owner (dynamic allocation).
7. Different compilation targets may implement these semantics differently (GC, reference counting, manual management). The source-level semantics are target-independent.

## What the Programmer Expresses (PROPOSED)

The programmer expresses three concepts:

### 1. Ownership Transfer (LOCKED requirement)

When a value is passed to a function or assigned to another binding, ownership transfers. The original binding can no longer used. This is the default behavior.

The exact syntax for expressing ownership transfer is PROPOSED. The semantic requirement (the value moves, the old binding is invalidated) is LOCKED.

### 2. Borrowing (LOCKED requirement)

The programmer can create references to values without taking ownership. References come in two forms:

- **Shared references**: Multiple readers, no writer. The value cannot be modified while shared references exist.
- **Exclusive reference**: One writer, no readers. The value can be modified. No other references (shared or exclusive) can exist while an exclusive reference exists.

The exact syntax for borrowing is PROPOSED. The semantic requirements (shared vs exclusive, mutual exclusion) are LOCKED.

### 3. Cleanup (LOCKED requirement)

Values are cleaned up when they go out of scope. Types can define custom cleanup behavior. The cleanup happens automatically; the programmer does not call cleanup explicitly.

The exact syntax for defining custom cleanup is PROPOSED. The semantic requirement (automatic cleanup at scope exit) is LOCKED.

## What the Programmer Does NOT Express

The following are NOT part of the source-level semantics:

- How references are implemented (pointers, indices, handles)
- Whether memory is garbage-collected, reference-counted, or manually managed
- Lifetime annotation syntax (this is a source representation choice, not a semantic requirement)
- Stack vs heap placement (the backend decides)
- Memory layout details (the backend decides)

This separation is deliberate. The same AXIOM source code must compile to different backend implementations without source changes.

## Backend Implementation (PROPOSED)

| Target | Implementation Strategy | Rationale |
|--------|------------------------|-----------|
| VM (MVP) | Tracing garbage collector | Simple to implement, correct, no programmer burden |
| Native | Reference counting (initially) | Deterministic cleanup, no stop-the-world pauses |
| WASM | Linear memory + bump/region allocator | WASM has no native GC |

The backend choice is independent of the source-level semantics. The programmer writes the same code regardless of target. The compiler selects the implementation strategy.

## Why This Separation

In Rust, the source-level ownership model is tightly coupled to the borrow checker and lifetimes, which are tightly coupled to the reference representation (raw pointers). This is correct for Rust's goals but creates a coupling that AXIOM does not need.

AXIOM's goals include targeting both systems and application domains. The source-level semantics (ownership, borrowing, cleanup) are expressive enough to describe memory safety properties. The backend implements these semantics using whatever mechanism is appropriate for the target.

This means:
- On the VM target, the GC handles most memory management. Ownership transfer and borrowing are still expressed in the source but the GC ensures no use-after-free.
- On the native target, reference counting provides deterministic cleanup. Ownership transfer decrements the reference count. Borrowing creates temporary references.
- On the WASM target, linear memory with regions provides allocation. The type system ensures safety.

The programmer does not need to know which backend is used. The semantics are the same.

## Destructor Protocol (PROPOSED)

When a value goes out of scope:

1. If the type defines a custom cleanup function, it is called
2. All fields are cleaned up in reverse declaration order
3. Memory is reclaimed (by whatever mechanism the backend uses)

The programmer can define custom cleanup for types that manage resources (files, network connections, locks). The exact syntax is PROPOSED.

## Why Not Pure GC

Pure garbage collection satisfies requirement 1 (predictable cleanup) but violates requirement 3 (the compiler cannot easily detect use-after-free or data races with a tracing GC, since values can be referenced from anywhere). A GC-only approach also makes it harder to satisfy requirement 4 (ownership transfer) in a way that has runtime meaning.

## Why Not Pure Manual Management

Pure manual management (C-style) satisfies all requirements but places excessive burden on the programmer. Every allocation must be matched with a deallocation. Every reference must be manually validated. This violates the AXIOM principle that high-level abstractions should remove unnecessary complexity.

## Why Not Lock Rust-Style Lifetimes

Rust's lifetime system is a proven mechanism for enforcing rules 2, 3, 4, and 5. However:

1. Lifetime annotations are a source representation choice, not a semantic requirement. The semantic requirements can be satisfied by other mechanisms.
2. Full lifetime enforcement creates significant ergonomic friction for application code. AXIOM targets both systems and application domains.
3. The VM target uses a GC, which makes lifetime annotations unnecessary for most code. Lifetime annotations on a GC target would be noise.
4. Lifetime enforcement can be added incrementally. The semantic model supports it. The implementation can start relaxed and tighten over time.

The LOCKED requirement is that the type system prevents data races and use-after-free "when feasible." This leaves room for the VM target to use GC (which handles these at runtime) while the native target uses more aggressive compile-time enforcement.

## Future: Region-Based Allocation (OPEN)

For performance-critical code, region-based allocation may be supported. All allocations in a region are freed at once when the region is dropped. This is an OPEN research question: the mechanism is proven (Rust's arena, Zig's allocator interface) but the source-level representation in AXIOM is not yet designed.
