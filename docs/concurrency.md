# AXIOM Concurrency Model (Revised)

## Semantic Requirements (LOCKED)

1. The language must support concurrent execution of multiple computations.
2. The programmer must be able to express that a computation runs concurrently with the current flow.
3. The programmer must be able to wait for a concurrent computation to produce a result.
4. Communication between concurrent computations must be explicit (not shared memory by default).
5. The type system must prevent data races at compile time when feasible.
6. Concurrent computations must be cancellable when their result is no longer needed.
7. Concurrent tasks must be structured: they have a lifetime bounded by their spawning scope.

## What the Programmer Expresses (PROPOSED)

### Task Spawning (LOCKED requirement, PROPOSED syntax)

The programmer can express that a computation should run concurrently. The spawned computation produces a value that can be received later.

The exact syntax for spawning and receiving is PROPOSED. The semantic requirements (concurrent execution, result reception, structured lifetime) are LOCKED.

### Channels (LOCKED requirement, PROPOSED syntax)

Tasks communicate through typed channels. A channel carries values of a specific type. Sending and receiving are explicit operations.

The exact syntax for channel creation, sending, and receiving is PROPOSED. The semantic requirements (typed channels, explicit communication) are LOCKED.

### Structured Concurrency (LOCKED)

When a scope exits, all tasks spawned within that scope must either complete or be cancelled. The programmer does not need to manually track task lifetimes.

This is the structured concurrency constraint. It prevents task leaks and ensures predictable cleanup.

## Proposed Representation (PROPOSED)

The following is an illustration of the semantic requirements. The exact syntax is NOT locked.

- A concurrency primitive spawns a task that returns a handle
- The handle can be waited on to receive the result
- A channel type carries typed values between tasks
- Tasks are scoped to their spawning context

## Why Not Threads

OS threads are heavy (1MB+ stack each, expensive context switching). AXIOM tasks are lightweight, scheduled by the runtime. On native targets, the runtime maps tasks to a thread pool. On WASM, tasks map to the event loop.

## Why Not Actor Model

The actor model (Erlang, Akka) creates complexity in type tracking and message schema evolution. AXIOM uses structured concurrency with channels as the primitive. Actors can be built on top of tasks + channels.

## Why Not Go-Style CSP

Go's goroutines are not structured (they can outlive their spawner). There is no built-in mechanism for task cancellation. Channel direction is not fully type-checked. AXIOM's tasks are structured, cancellable, and channels are fully typed.

## Task Lifecycle (PROPOSED)

Tasks have a lifecycle: running, completed, cancelled, failed.

- Running: Task is executing
- Completed: Task returned a value
- Cancelled: Task was cancelled (parent scope exited)
- Failed: Task encountered an unrecoverable error

When a scope exits, child tasks are cancelled or awaited. This is the structured concurrency constraint (LOCKED).

## Synchronization Primitives (PROPOSED)

For cases where shared state is necessary (rare), the language provides:

- Atomic operations for simple values
- Mutex for mutual exclusion
- RwLock for reader-writer locking

These are built into the runtime, not the standard library.

## Concurrency by Target

| Target | Task Implementation | Scheduling |
|--------|-------------------|------------|
| Native | OS threads + green tasks | Work-stealing thread pool |
| WASM | Web Workers + message passing | Browser event loop |
| VM | Green tasks | Cooperative scheduler |

The concurrency semantics are the same across targets. The runtime handles the mapping.

## Why Not Effect-Based Concurrency

Effect systems can track I/O and mutation at the type level. This is an OPEN research question for AXIOM. Effect-based concurrency would enable compile-time verification of concurrent safety properties but adds significant complexity. Deferred from MVP.
