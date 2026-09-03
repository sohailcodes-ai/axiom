# AXIOM Execution Model (Revised)

## Semantic Requirements (LOCKED)

1. An AXIOM program contains code that executes in different environments with different capabilities.
2. The compiler must know which environment each function executes in.
3. The compiler must verify that a function only uses operations available in its declared environment.
4. When a value crosses from one environment to another, the compiler must verify the value is transmissible.
5. The programmer must be able to see, from the source code, which environment a function executes in.
6. Cross-environment calls are not direct function calls - they are message exchanges.

## Execution Domains

An execution domain is a named compilation boundary. The compiler partitions the program by domain. Each domain becomes a separate compilation unit that targets a different output format.

### Domain Definitions

| Domain | Capabilities | Restrictions |
|--------|-------------|--------------|
| server | Filesystem, networking, databases, process spawning, full system access | No DOM, no browser APIs |
| client | DOM, fetch, localStorage, browser APIs | No filesystem, no process spawning, no raw memory access |
| worker | Computation, message passing, limited I/O | No DOM, no direct system calls |
| native | Raw memory, system calls, FFI, hardware access | No automatic memory safety - explicit management required |

### Proposed Source Representation (PROPOSED)

The exact syntax for declaring execution domains is unresolved. This is a PROPOSED direction, not a locked decision. Candidates include:

**Block-level domains:**
```
domain server {
    fn handle_request() -> Response { ... }
}
```

**Function-level annotations:**
```
@server fn handle_request() -> Response { ... }
```

**Position markers:**
```
fn handle_request() server -> Response { ... }
```

The syntax is PROPOSED. The semantic requirements above are LOCKED regardless of which syntax is chosen.

### Cross-Domain Communication

Cross-domain calls produce serialized messages. The compiler generates serialization code. Values crossing boundaries must satisfy a transmissibility constraint.

The transmissibility constraint is an OPEN research question. The semantic requirement (values crossing boundaries must be transmissible) is LOCKED. The definition of which types are transmissible is OPEN.

### Compilation Units

The compiler partitions the program by domain. Each domain becomes a separate compilation unit:

```
AXIOM Program
  -> Server compilation unit -> VM bytecode (server)
  -> Client compilation unit -> VM bytecode (client)
  -> Worker compilation unit -> VM bytecode (worker)
  -> [Future: native binary, WASM module]
```

This partitioning happens after type checking. The typed AST is divided by domain declarations, and each partition is lowered and compiled independently.

### Domain Analysis Stage (LOCKED stage)

Domain analysis is a distinct compiler stage that runs after type checking. It verifies:

- Each function operates only within its declared domain's capabilities
- Cross-domain calls are properly routed through the message protocol
- Values crossing domain boundaries are transmissible
- No domain-forbidden operations are used

This stage does not exist in other languages. It is the compiler-verified typed execution boundary.

### AXIOM Differentiator: Compiler-Verified Typed Execution Boundaries

This is AXIOM's primary language-level differentiator.

In a typical full-stack application today (e.g., TypeScript + Express), nothing prevents server-only code (database queries, filesystem access) from accidentally ending up in a client bundle. The error is caught at runtime, or by a linter with limited understanding.

In AXIOM, the compiler verifies this at compile time. The compiler knows:

1. Which domain each function belongs to
2. Which domains each function calls
3. Which types cross domain boundaries
4. Whether those types are transmissible

This is verified at compile time, not runtime, not by a linter. It is a type-system property.

**Why this is different from existing languages:**

- TypeScript: No domain awareness. Client/server separation is manual.
- Rust: Ownership tracks memory, not execution domains.
- Go: No built-in client/server distinction.
- Swift: Actors track concurrency, not deployment domains.
- Kotlin: Coroutines track async, not deployment domains.
- Elm: Client-only, no server domain.
- PureScript: Compiles to JS, no domain awareness.

AXIOM tracks where code runs as a type-system property. This combination does not exist in any mainstream language.

### Concurrency Within Domains

Within a single domain, AXIOM supports concurrent execution through structured tasks and message passing. See the concurrency model for details.

### Task Model (PROPOSED)

Tasks are lightweight units of computation. They are spawned within a domain and communicate through typed channels. Tasks are structured: they have a clear lifetime bound to their spawning scope.

When a scope exits, child tasks are cancelled or awaited. This is the structured concurrency constraint (LOCKED).

### Execution Flow

1. The compiler parses and type-checks the entire program
2. The compiler performs domain analysis (verifies boundaries)
3. The compiler partitions by domain
4. Each domain is compiled to its target representation
5. At runtime, the appropriate domain is loaded for each environment
6. Tasks within each domain are scheduled by the runtime

### Why This Design

The alternative (a single execution domain, like Node.js) hides where code runs. This creates performance surprises: a function that looks cheap might do a network call. Another alternative (completely separate languages) breaks the one-language principle.

AXIOM's approach makes execution boundaries explicit and type-checked. The programmer declares where code runs. The compiler verifies it. The runtime enforces it.
