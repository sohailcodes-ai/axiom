# AXIOM Runtime

## Overview

The AXIOM runtime is the minimal execution environment required to run AXIOM programs. It is not a standard library — it is the bare minimum to execute code.

## Runtime Components

### 1. Task Scheduler

Manages lightweight tasks (green threads). Schedules tasks across OS threads (native) or cooperatively (VM/WASM).

**Responsibilities:**
- Task creation and destruction
- Task scheduling (work-stealing on native, cooperative on VM)
- Task cancellation
- Async/await state machine management

### 2. Garbage Collector

Manages heap-allocated values. Implementation varies by target:

| Target | GC Strategy |
|--------|-------------|
| VM | Tracing GC (mark-and-sweep) |
| Native | Reference counting (initially) |
| WASM | Custom allocator (bump + GC) |

**GC Interface:**

```rust
trait Gc {
    fn alloc(&mut self, size: usize) -> *mut u8;
    fn collect(&mut self);
    fn stats(&self) -> GcStats;
}
```

### 3. Channel Implementation

Typed message channels for inter-task communication.

**Channel Interface:**

```rust
trait Channel<T: Serializable> {
    fn send(&self, value: T);
    async fn receive(&self) -> T;
    fn try_receive(&self) -> Option<T>;
    fn clone(&self) -> Self;
}
```

### 4. I/O Reactor

Event-driven I/O for async operations.

**Native:** epoll (Linux), kqueue (macOS), IOCP (Windows)
**VM:** Host function calls
**WASM:** Browser event loop or WASI

### 5. Memory Allocator

Configurable allocator interface:

```rust
trait Allocator {
    fn alloc(&self, layout: Layout) -> *mut u8;
    fn dealloc(&self, ptr: *mut u8, layout: Layout);
}
```

Default allocators:
- Native: System allocator (jemalloc as optional)
- VM: Arena allocator + GC
- WASM: Bump allocator

## Runtime Size Constraints

| Target | Target Size |
|--------|-------------|
| VM | < 200KB compiled |
| Native | < 500KB compiled |
| WASM | < 300KB compiled |

## Runtime Linking

The runtime is statically linked into every AXIOM program. There is no dynamic runtime dependency. This is similar to Go's approach (runtime is linked into every binary) and opposite to Java's approach (JVM is a separate dependency).

## Host Functions

The runtime exposes operations to AXIOM code through host functions. These are the primitive operations that AXIOM code can call:

```rust
// I/O
fn host_print(value: &Value);
fn host_read_file(path: &str) -> Result<Vec<u8>, IoError>;
fn host_write_file(path: &str, data: &[u8]) -> Result<(), IoError>;

// Networking
fn host_http_get(url: &str) -> Result<Response, NetworkError>;
fn host_http_post(url: &str, body: &[u8]) -> Result<Response, NetworkError>;

// Tasks
fn host_spawn(entry: FunctionId) -> TaskHandle;
fn host_await(handle: TaskHandle) -> Value;
fn host_channel_create() -> ChannelHandle;
fn host_channel_send(handle: ChannelHandle, value: &Value);
fn host_channel_receive(handle: ChannelHandle) -> Value;

// Time
fn host_now() -> u64;  // Milliseconds since epoch
fn host_sleep(ms: u64);

// System
fn host_args() -> Vec<String>;
fn host_env(key: &str) -> Option<String>;
```

Host functions are the boundary between AXIOM code and the runtime. They are the only way AXIOM code can perform I/O or interact with the system.

## Runtime Initialization

When an AXIOM program starts:

1. Runtime is initialized (memory, GC, task scheduler)
2. Command-line arguments are parsed
3. Entry point function is called
4. Main task runs
5. When main task completes, runtime waits for remaining tasks (with timeout)
6. Runtime shuts down (GC finalization, resource cleanup)

## Runtime Configuration

The runtime can be configured at startup:

```axiom
@config
fn runtime_config() -> Config {
    return Config {
        gc_strategy: GcStrategy::Generational,
        task_pool_size: 8,
        gc_heap_growth: 1.5,  // 50% growth factor
    };
}
```

This is not in the MVP. The MVP uses default configuration.

## Why a Custom Runtime

Existing runtimes (JVM, Node.js, CLR) are heavy and impose their own semantics. AXIOM needs a runtime that:
- Is small (fits in embedded contexts)
- Supports AXIOM's concurrency model
- Has no external dependencies
- Can be cross-compiled easily
- Is AXIOM-specific (no legacy baggage)

The runtime is written in Rust for the same reasons the compiler is: memory safety, good tooling, no runtime dependency.
