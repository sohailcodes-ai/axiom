# AXIOM Philosophy

## What AXIOM Is

AXIOM is a general-purpose, statically-typed programming language designed for building software across the full stack: CLI applications, native applications, backend services, web frontends, distributed systems, and WASM modules — all from a single language with a single type system.

## What AXIOM Is Not

- Not a web-only language
- Not a framework
- Not a tutorial project
- Not a JavaScript replacement
- Not a Rust clone
- Not a language that requires a paid service to use

## Core Principles

### 1. General Purpose

AXIOM must be capable of expressing any computation a programmer needs. It is not optimized for one domain at the expense of others. A language optimized only for web development is not general-purpose. A language optimized only for systems programming is not general-purpose.

### 2. Full Stack

A single program can contain client code, server code, worker code, and native code. The programmer writes one program. The compiler understands the boundaries and generates appropriate output for each context.

### 3. One Language, One Type System

There is no separate "frontend language" and "backend language." Types are coherent across the entire application boundary. A `User` type defined once is the same `User` type on the client and the server.

### 4. Developer Control

High-level abstractions exist to remove unnecessary complexity. Low-level control exists to handle cases where the abstraction leaks. The programmer chooses when to use which.

### 5. Explicit Over Implicit

Execution boundaries, memory allocation, error handling, and concurrency are explicit. Magic is avoided. If something happens, the programmer can see it in the source code.

### 6. Zero Mandatory Cost

The core language, compiler, runtime, and local development workflow are free. No paid accounts, no paid APIs, no proprietary services required to write and run AXIOM code.

### 7. AI Is Optional

AI capabilities are a feature some applications may use, not a requirement of the language. If an application uses AI, the developer brings their own provider credentials (BYOK — Bring Your Own Key).

### 8. No Framework Dependency

AXIOM does not require React, Node, Express, Django, Spring, or any external framework to be useful. The standard library provides sufficient primitives. Frameworks may exist but are not mandatory.

### 9. First-Class Concurrency

Concurrency, async, parallelism, workers, messaging, and realtime are fundamental language and runtime concerns, not afterthoughts bolted onto a sequential language.

### 10. Measurable, Not Marketable

AXIOM does not make unsupported claims about performance, safety, or productivity. Everything is eventually measurable. Benchmarks exist. Safety properties are formalized. Productivity is measured by real-world usage.

## Design Heuristics

When facing a design choice:

1. **Correctness first.** If a design makes it easy to write incorrect code, reject it.
2. **Simplicity second.** If two designs are equally correct, choose the simpler one.
3. **Performance third.** If two designs are equally correct and simple, choose the faster one.
4. **Ergonomics fourth.** If two designs are equally correct, simple, and fast, choose the more pleasant one.

These heuristics are ordered. Correctness always wins over ergonomics. But ergonomics matters — a correct language that nobody wants to use is a failed language.
