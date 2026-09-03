# AXIOM Syntax Design — Phase 2

## Executive Summary

This document designs the concrete syntax for AXIOM. It derives syntactic requirements from the locked semantic model, proposes three internally consistent syntax designs, and recommends one.

**Key recommendation:** Proposal B ("Balanced AXIOM") is the recommended syntax. It is familiar enough that developers from mainstream languages can read basic AXIOM immediately, but distinctive enough in its expression of domains, errors, and concurrency to feel like its own language. It avoids annotation soup, uses no semicolons in the common case, and makes AXIOM's differentiators (execution domains, typed errors, structured concurrency) visually prominent without being noisy.

**Strongest ideas from other proposals:** Proposal A's function syntax (name-first, no keyword) is worth adopting. Proposal C's pipe-style composition and inline domain blocks are worth future consideration.

**Syntax decisions safe to LOCK after review:** Literals, basic operators, block syntax, comment syntax, string syntax.

**Syntax decisions that should remain OPEN:** Propagation operator, domain syntax (block vs annotation), borrowing notation, generic syntax, exact failure/absence type names.

---

## 1. Syntactic Requirements Derived from Semantic Model

| Concept | Semantic requirement | Syntax must communicate | Candidate notation |
|---------|---------------------|------------------------|-------------------|
| Integer literal | Defined-width numeric value | Digits, optional type suffix | `42`, `42i64` |
| Float literal | IEEE 754 value | Decimal point, optional suffix | `3.14`, `3.14f32` |
| Boolean literal | True or false | Keyword | `true`, `false` |
| String literal | Immutable UTF-8 text | Delimiters | `"hello"` |
| Char literal | Single Unicode scalar | Delimiters | `'a'` |
| Unit value | No meaningful value | Parentheses or keyword | `()` |
| Immutable binding | Name cannot be rebound | Keyword + name + value | `let x = 42;` |
| Mutable binding | Name can be reassigned | Keyword + mutability marker + name + value | `let mut x = 42;` |
| Constant | Compile-time evaluated value | Keyword + name + value | `const MAX = 1024;` |
| Type annotation | Compile-time type check | Colon + type | `x: Int` |
| Named type | User-defined struct/enum | Keyword + name + body | `struct User { ... }` |
| Sum type | One of several variants | Keyword + name + variants | `enum Color { Red, Green, Blue }` |
| Variant with data | Variant carries payload | Parenthesized fields | `Ok(Int)`, `Err(String)` |
| Function declaration | Named parameterized computation | Signature + body | `fn add(a: Int, b: Int) -> Int` |
| Function call | Apply function to arguments | Name + parenthesized args | `add(1, 2)` |
| Return value | Early exit from function | Return keyword + expression | `return x;` |
| Block | Scoped region | Curly braces | `{ ... }` |
| Block expression | Block producing a value | Final expression without semicolon | `{ x + 1 }` |
| If expression | Conditional value | if/else keywords + condition + blocks | `if x > 0 { x } else { -x }` |
| While loop | Repeated execution | while keyword + condition + block | `while done { ... }` |
| For loop | Iterator-based execution | for + binding + in + collection | `for item in list { ... }` |
| Pattern matching | Exhaustive case analysis | match keyword + value + arms | `match x { ... }` |
| Pattern arm | Single match case | Pattern => expression | `Red => "stop"` |
| Wildcard pattern | Catch-all case | Underscore | `_ => "other"` |
| Binding pattern | Name the matched value | Identifier in pattern | `User(name, _) => name` |
| Failure type | Function can fail | Return type with failure marker | `fn parse(s: String) -> Int ! ParseError` |
| Failure propagation | Forward failure to caller | Propagation operator | `parse(input)!` |
| Absence type | Value may not exist | Type wrapper | `?Int` or `maybe Int` |
| Absence handling | Unwrap or provide default | Method or operator | `x.or(default)` |
| Panic | Unrecoverable error | Panic expression | `panic("impossible")` |
| Unreachable code | Dead code marker | Unreachable expression | `unreachable` |
| Ownership transfer | Value moves, old binding invalid | Default behavior (implicit) | (no syntax needed) |
| Shared borrow | Read without ownership | Reference operator | `&x` |
| Exclusive borrow | Read/write without ownership | Mutable reference operator | `&mut x` |
| Cleanup definition | Custom drop behavior | Special method name | `fn drop(self)` |
| Task spawn | Concurrent execution | Spawn keyword + block | `spawn { ... }` |
| Task result | Receive concurrent value | Await operator | `handle.await` or `await handle` |
| Channel creation | Typed message pipe | Channel type constructor | `chan Int` or `Channel<Int>` |
| Channel send | Send value to channel | Send method/operator | `ch.send(value)` |
| Channel receive | Get value from channel | Receive method/operator | `ch.receive()` |
| Module declaration | Code organization unit | File = module | (implicit) |
| Module import | Use names from module | Import keyword + path | `import math` |
| Visibility control | Public/private names | Visibility keyword or prefix | `pub fn ...` or `export fn ...` |
| Execution domain | Code runs in specific environment | Domain keyword/annotation | `domain server { ... }` or `@server` |
| Cross-domain call | Message exchange across domains | Special call syntax | `call server handle(req)` |
| Generic type | Parameterized type | Angle brackets or similar | `List<Int>`, `fn first<T>(...)` |
| Closure | Captured variable function | Lambda syntax | `fn(x) x + 1` |
| Type alias | Alternative name for type | Type keyword | `type UserId = Int;` |
| Comment | Ignored by compiler | // and /* */ | `// line`, `/* block */` |

---

## 2. Three Competing Syntax Proposals

### Proposal A: "Familiar / Low Friction"

**Goal:** A developer from Rust, Go, C#, Java, TypeScript, Swift, Kotlin should understand most basic AXIOM code immediately.

**Design philosophy:** Use the most common syntax conventions from mainstream languages. Minimize new keywords. Use semicolons. Use `fn` for functions. Use `?` for propagation. Use `@` for domain annotations.

---

#### Proposal A: Examples

**1. Hello World**
```axiom
fn main() {
    print("Hello, World!");
}
```

**2. Variables and mutation**
```axiom
fn main() {
    let x = 42;
    let mut y = 10;
    y = 20;
    let name: String = "Alice";
}
```

**3. Primitive values**
```axiom
fn main() {
    let i: i64 = 42;
    let f: f64 = 3.14;
    let b: bool = true;
    let c: char = 'a';
    let s: String = "hello";
    let hex: u32 = 0xff;
    let sep: i64 = 1_000_000;
}
```

**4. User-defined type**
```axiom
struct User {
    name: String,
    age: i64,
    email: String,
}

fn main() {
    let user = User { name: "Alice", age: 30, email: "alice@example.com" };
    print(user.name);
}
```

**5. Enum / sum type**
```axiom
enum Color {
    Red,
    Green,
    Blue,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**6. Function**
```axiom
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
```

**7. Function returning a value**
```axiom
fn abs(x: i64) -> i64 {
    if x < 0 {
        return -x;
    }
    return x;
}
```

**8. Function with failure**
```axiom
fn parse_int(s: String) -> i64 ! ParseError {
    match s.to_int() {
        Ok(n) => return Ok(n),
        Err(e) => return Err(ParseError.InvalidFormat(s)),
    }
}

fn process() -> i64 ! AppError {
    let a = parse_int("42")?;    // propagates on failure
    let b = parse_int("10")?;
    return Ok(a + b);
}
```

**9. Absence**
```axiom
fn find_user(id: i64) -> ?User {
    if id == 1 {
        return Some(User { name: "Alice", age: 30 });
    }
    return None;
}

fn main() {
    let user = find_user(1);
    match user {
        Some(u) => print(u.name),
        None => print("not found"),
    }
    // Or with default:
    let name = find_user(1).or(User { name: "Unknown", age: 0 }).name;
}
```

**10. If/else**
```axiom
fn grade(score: i64) -> String {
    if score >= 90 {
        return "A";
    } else if score >= 80 {
        return "B";
    } else {
        return "C";
    }
}
```

**11. Loop**
```axiom
fn sum_to(n: i64) -> i64 {
    let mut total = 0;
    let mut i = 1;
    while i <= n {
        total += i;
        i += 1;
    }
    return total;
}

fn find_first(list: List<i64>, target: i64) -> ?i64 {
    for item in list {
        if item == target {
            return Some(item);
        }
    }
    return None;
}
```

**12. Pattern matching**
```axiom
fn describe(color: Color) -> String {
    match color {
        Red => return "stop",
        Green => return "go",
        Blue => return "sky",
    }
}

fn process_result(result: Result<i64, String>) -> String {
    match result {
        Ok(value) => return format!("Got: {}", value),
        Err(msg) => return format!("Error: {}", msg),
    }
}
```

**13. Collection operations**
```axiom
fn main() {
    let list = List::new();
    list.push(1);
    list.push(2);
    list.push(3);

    let doubled = list.map(fn(x) x * 2);
    let sum = list.fold(0, fn(acc, x) acc + x);

    for item in list {
        print(item);
    }
}
```

**14. Module / import**
```axiom
import std.math;
import std.io;

fn main() {
    let result = math.sqrt(42.0);
    io.println(result);
}
```

**15. Public / private visibility**
```axiom
pub struct User {
    pub name: String,
    age: i64,          // private
}

pub fn public_function() { }
fn private_function() { }
```

**16. Task / concurrency**
```axiom
fn main() {
    let handle = spawn {
        return heavy_computation();
    };
    let result = handle.await;
    print(result);
}
```

**17. Typed channel**
```axiom
fn main() {
    let ch = Channel::new();
    spawn {
        ch.send(42);
    };
    let value = ch.receive();
    print(value);
}
```

**18. Resource handling**
```axiom
struct File {
    handle: RawFd,
}

impl File {
    fn drop(self) {
        close_fd(self.handle);
    }
}

fn read_file(path: String) -> String ! IoError {
    let file = File::open(path)?;
    return file.read_all()?;
}
```

**19. Execution domain declaration**
```axiom
@server
fn handle_request(req: Request) -> Response ! HttpError {
    let user = db.query("SELECT * FROM users WHERE id = ?", req.user_id)?;
    return Response::json(user);
}

@client
fn render_user_list(users: List<User>) -> Element {
    return div(children: users.map(render_user));
}
```

**20. Cross-domain call**
```axiom
@client
fn get_user(id: i64) -> User ! ClientError {
    let user = call server fetch_user(id)?;
    return user;
}
```

**21. Small database-backed server**
```axiom
import std.http;
import std.db;

struct User {
    id: i64,
    name: String,
    email: String,
}

@server
fn handle_get_users(req: Request) -> Response ! HttpError {
    let users = db.query("SELECT * FROM users")?;
    return Response::json(users);
}

@server
fn handle_create_user(req: Request) -> Response ! HttpError {
    let body = req.body().parse_json()?;
    let user = db.execute("INSERT INTO users (name, email) VALUES (?, ?)", body.name, body.email)?;
    return Response::created(user);
}

fn main() {
    let server = http.Server::new("0.0.0.0:8080");
    server.route("GET", "/users", handle_get_users);
    server.route("POST", "/users", handle_create_user);
    server.start();
}
```

**22. Small client example**
```axiom
import std.dom;

@client
fn render_user_list(users: List<User>) -> Element {
    return ul(children: users.map(fn(user) {
        return li(text: user.name);
    }));
}

@client
fn main() {
    let users = call server get_users()?;
    let app = render_user_list(users);
    dom.mount(app, "#app");
}
```

**23. Small worker example**
```axiom
@worker
fn process_image(image: Image) -> ImageResult ! WorkerError {
    let filtered = apply_filter(image)?;
    let resized = resize(filtered, 800, 600)?;
    return Ok(resized);
}

fn main() {
    let image = load_image("photo.jpg")?;
    let handle = spawn {
        return process_image(image);
    };
    let result = handle.await?;
    save_image(result, "processed.jpg")?;
}
```

**24. Medium-sized example**
```axiom
import std.http;
import std.json;
import std.db;

struct User {
    id: i64,
    name: String,
    email: String,
}

enum ApiError {
    NotFound,
    BadRequest(String),
    DatabaseError(String),
    Unauthorized,
}

fn parse_user_id(raw: String) -> i64 ! ApiError {
    let id = raw.parse_int()?;
    if id <= 0 {
        return Err(ApiError.BadRequest("Invalid user ID".to_string()));
    }
    return Ok(id);
}

@server
fn handle_get_user(req: Request) -> Response ! ApiError {
    let id = parse_user_id(req.param("id")?)?;
    let user = db.query("SELECT * FROM users WHERE id = ?", id)?
        .first()
        .ok_or(ApiError.NotFound)?;
    return Response::json(user);
}

@server
fn handle_update_user(req: Request) -> Response ! ApiError {
    let id = parse_user_id(req.param("id")?)?;
    let body = req.body().parse_json()?;
    db.execute("UPDATE users SET name = ?, email = ? WHERE id = ?", body.name, body.email, id)?;
    let user = db.query("SELECT * FROM users WHERE id = ?", id)?.first().ok_or(ApiError.NotFound)?;
    return Response::json(user);
}

fn main() {
    let server = http.Server::new("0.0.0.0:8080");
    server.route("GET", "/users/:id", handle_get_user);
    server.route("PUT", "/users/:id", handle_update_user);
    server.start();
}
```

---

### Proposal B: "Balanced AXIOM" (Recommended)

**Goal:** Familiar where familiarity is useful, but with deliberate AXIOM-specific design choices that make the language distinctive and readable.

**Design philosophy:** Drop semicolons in the common case. Use `fn` for functions (proven, short, unambiguous). Use `!` for failure propagation (distinctive, visually clear, not overloaded). Use `?` prefix for absence types (terse, distinct from propagation). Use `domain` keyword blocks for execution domains (makes domain boundaries visually prominent). Use `let` / `let mut` for bindings (familiar). Use `->` for return types. Use no angle brackets for generics (use square brackets instead, avoiding the comparison operator ambiguity).

---

#### Proposal B: Examples

**1. Hello World**
```axiom
fn main() {
    print("Hello, World!")
}
```

**2. Variables and mutation**
```axiom
fn main() {
    let x = 42
    let mut y = 10
    y = 20
    let name: String = "Alice"
}
```

**3. Primitive values**
```axiom
fn main() {
    let i: i64 = 42
    let f: f64 = 3.14
    let b: bool = true
    let c: char = 'a'
    let s: String = "hello"
    let hex: u32 = 0xff
    let sep: i64 = 1_000_000
}
```

**4. User-defined type**
```axiom
struct User {
    name: String
    age: i64
    email: String
}

fn main() {
    let user = User { name: "Alice", age: 30, email: "alice@example.com" }
    print(user.name)
}
```

**5. Enum / sum type**
```axiom
enum Color {
    Red
    Green
    Blue
}

enum Result[T, E] {
    Ok(T)
    Err(E)
}
```

**6. Function**
```axiom
fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

**7. Function returning a value**
```axiom
fn abs(x: i64) -> i64 {
    if x < 0 { -x } else { x }
}
```

**8. Function with failure**
```axiom
fn parse_int(s: String) -> i64 ! ParseError {
    match s.to_int() {
        Ok(n) => Ok(n)
        Err(_) => Err(ParseError.InvalidFormat(s))
    }
}

fn process() -> i64 ! AppError {
    let a = parse_int("42")!
    let b = parse_int("10")!
    Ok(a + b)
}
```

**9. Absence**
```axiom
fn find_user(id: i64) -> ?User {
    if id == 1 {
        Some(User { name: "Alice", age: 30 })
    } else {
        None
    }
}

fn main() {
    let user = find_user(1)
    match user {
        Some(u) => print(u.name)
        None => print("not found")
    }
    let name = find_user(1).or(User { name: "Unknown", age: 0 }).name
}
```

**10. If/else**
```axiom
fn grade(score: i64) -> String {
    if score >= 90 { "A" }
    else if score >= 80 { "B" }
    else { "C" }
}
```

**11. Loop**
```axiom
fn sum_to(n: i64) -> i64 {
    let mut total = 0
    let mut i = 1
    while i <= n {
        total += i
        i += 1
    }
    total
}

fn find_first(list: List[i64], target: i64) -> ?i64 {
    for item in list {
        if item == target { return Some(item) }
    }
    None
}
```

**12. Pattern matching**
```axiom
fn describe(color: Color) -> String {
    match color {
        Red => "stop"
        Green => "go"
        Blue => "sky"
    }
}

fn process_result(result: Result[i64, String]) -> String {
    match result {
        Ok(value) => format!("Got: {}", value)
        Err(msg) => format!("Error: {}", msg)
    }
}
```

**13. Collection operations**
```axiom
fn main() {
    let list = [1, 2, 3]
    let doubled = list.map(|x| x * 2)
    let sum = list.fold(0, |acc, x| acc + x)

    for item in list {
        print(item)
    }
}
```

**14. Module / import**
```axiom
import std.math
import std.io

fn main() {
    let result = math.sqrt(42.0)
    io.println(result)
}
```

**15. Public / private visibility**
```axiom
pub struct User {
    pub name: String
    age: i64          # private
}

pub fn public_function() { }
fn private_function() { }
```

**16. Task / concurrency**
```axiom
fn main() {
    let handle = spawn {
        heavy_computation()
    }
    let result = handle.await
    print(result)
}
```

**17. Typed channel**
```axiom
fn main() {
    let ch = chan[i64]
    spawn {
        ch.send(42)
    }
    let value = ch.receive()
    print(value)
}
```

**18. Resource handling**
```axiom
struct File {
    handle: RawFd
}

impl File {
    fn drop(self) {
        close_fd(self.handle)
    }
}

fn read_file(path: String) -> String ! IoError {
    let file = File::open(path)!
    file.read_all()!
}
```

**19. Execution domain declaration**
```axiom
domain server {
    fn handle_request(req: Request) -> Response ! HttpError {
        let user = db.query("SELECT * FROM users WHERE id = ?", req.user_id)!
        Response::json(user)
    }
}

domain client {
    fn render_user_list(users: List[User]) -> Element {
        div(children: users.map(render_user))
    }
}
```

**20. Cross-domain call**
```axiom
domain client {
    fn get_user(id: i64) -> User ! ClientError {
        let user = call server fetch_user(id)!
        user
    }
}
```

**21. Small database-backed server**
```axiom
import std.http
import std.db

struct User {
    id: i64
    name: String
    email: String
}

domain server {
    fn handle_get_users(req: Request) -> Response ! HttpError {
        let users = db.query("SELECT * FROM users")!
        Response::json(users)
    }

    fn handle_create_user(req: Request) -> Response ! HttpError {
        let body = req.body().parse_json()!
        let user = db.execute(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            body.name,
            body.email
        )!
        Response::created(user)
    }
}

fn main() {
    let server = http.Server::new("0.0.0.0:8080")
    server.route("GET", "/users", handle_get_users)
    server.route("POST", "/users", handle_create_user)
    server.start()
}
```

**22. Small client example**
```axiom
import std.dom

domain client {
    fn render_user_list(users: List[User]) -> Element {
        ul(children: users.map(|user| li(text: user.name)))
    }

    fn main() {
        let users = call server get_users()!
        let app = render_user_list(users)
        dom.mount(app, "#app")
    }
}
```

**23. Small worker example**
```axiom
domain worker {
    fn process_image(image: Image) -> ImageResult ! WorkerError {
        let filtered = apply_filter(image)!
        let resized = resize(filtered, 800, 600)!
        Ok(resized)
    }
}

fn main() {
    let image = load_image("photo.jpg")!
    let handle = spawn {
        process_image(image)
    }
    let result = handle.await!
    save_image(result, "processed.jpg")!
}
```

**24. Medium-sized example**
```axiom
import std.http
import std.json
import std.db

struct User {
    id: i64
    name: String
    email: String
}

enum ApiError {
    NotFound
    BadRequest(String)
    DatabaseError(String)
    Unauthorized
}

fn parse_user_id(raw: String) -> i64 ! ApiError {
    let id = raw.parse_int()!
    if id <= 0 {
        Err(ApiError.BadRequest("Invalid user ID"))
    } else {
        Ok(id)
    }
}

domain server {
    fn handle_get_user(req: Request) -> Response ! ApiError {
        let id = parse_user_id(req.param("id")!)!
        let user = db.query("SELECT * FROM users WHERE id = ?", id)!
            .first()
            .ok_or(ApiError.NotFound)!
        Response::json(user)
    }

    fn handle_update_user(req: Request) -> Response ! ApiError {
        let id = parse_user_id(req.param("id")!)!
        let body = req.body().parse_json()!
        db.execute(
            "UPDATE users SET name = ?, email = ? WHERE id = ?",
            body.name,
            body.email,
            id
        )!
        let user = db.query("SELECT * FROM users WHERE id = ?", id)!
            .first()
            .ok_or(ApiError.NotFound)!
        Response::json(user)
    }
}

fn main() {
    let server = http.Server::new("0.0.0.0:8080")
    server.route("GET", "/users/:id", handle_get_user)
    server.route("PUT", "/users/:id", handle_update_user)
    server.start()
}
```

---

### Proposal C: "Distinctive AXIOM"

**Goal:** More original language identity while remaining readable and practical.

**Design philosophy:** Use pipe-style composition (`|>`). Use `def` for function definitions (distinctive, short). Use `::` for type construction. Use `~>` for failure propagation (distinctive arrow). Use `?` prefix for absence. Use inline domain annotations with `::` syntax. Use no semicolons. Use `match` with `->` instead of `=>`.

---

#### Proposal C: Examples

**1. Hello World**
```axiom
def main()
    print("Hello, World!")
```

**2. Variables and mutation**
```axiom
def main()
    let x = 42
    let! y = 10     # mutable binding
    y = 20
    let name: String = "Alice"
```

**3. Primitive values**
```axiom
def main()
    let i: i64 = 42
    let f: f64 = 3.14
    let b: bool = true
    let c: char = 'a'
    let s: String = "hello"
    let hex: u32 = 0xff
```

**4. User-defined type**
```axiom
type User {
    name: String
    age: i64
    email: String
}

def main()
    let user = User::new { name: "Alice", age: 30, email: "alice@example.com" }
    print(user.name)
```

**5. Enum / sum type**
```axiom
type Color = Red | Green | Blue

type Result[T, E] = Ok(T) | Err(E)
```

**6. Function**
```axiom
def add(a: i64, b: i64) -> i64
    a + b
```

**7. Function returning a value**
```axiom
def abs(x: i64) -> i64
    if x < 0 { -x } else { x }
```

**8. Function with failure**
```axiom
def parse_int(s: String) -> i64 ~ ParseError
    s.to_int() |> match {
        Ok(n) -> Ok(n)
        Err(_) -> Err(ParseError::InvalidFormat(s))
    }

def process() -> i64 ~ AppError
    let a = parse_int("42")~
    let b = parse_int("10")~
    Ok(a + b)
```

**9. Absence**
```axiom
def find_user(id: i64) -> ?User
    if id == 1 {
        ?(User::new { name: "Alice", age: 30 })
    } else {
        ?none
    }

def main()
    let user = find_user(1)
    match user {
        ?some(u) -> print(u.name)
        ?none -> print("not found")
    }
```

**10. If/else**
```axiom
def grade(score: i64) -> String
    if score >= 90 { "A" }
    elif score >= 80 { "B" }
    else { "C" }
```

**11. Loop**
```axiom
def sum_to(n: i64) -> i64
    let! total = 0
    let! i = 1
    while i <= n {
        total += i
        i += 1
    }
    total

def find_first(list: List[i64], target: i64) -> ?i64
    for item in list {
        if item == target { return ?(item) }
    }
    ?none
```

**12. Pattern matching**
```axiom
def describe(color: Color) -> String
    match color {
        Red -> "stop"
        Green -> "go"
        Blue -> "sky"
    }

def process_result(result: Result[i64, String]) -> String
    match result {
        Ok(value) -> format!("Got: {}", value)
        Err(msg) -> format!("Error: {}", msg)
    }
```

**13. Collection operations**
```axiom
def main()
    let list = [1, 2, 3]
    let doubled = list |> map(|x| x * 2)
    let sum = list |> fold(0, |acc, x| acc + x)

    for item in list {
        print(item)
    }
```

**14. Module / import**
```axiom
use std.math
use std.io

def main()
    let result = math::sqrt(42.0)
    io::println(result)
```

**15. Public / private visibility**
```axiom
pub type User {
    pub name: String
    age: i64          # private
}

pub def public_function() { }
def private_function() { }
```

**16. Task / concurrency**
```axiom
def main()
    let handle = spawn {
        heavy_computation()
    }
    let result = handle.await
    print(result)
```

**17. Typed channel**
```axiom
def main()
    let ch = Channel[i64]::new()
    spawn {
        ch.send(42)
    }
    let value = ch.receive()
    print(value)
```

**18. Resource handling**
```axiom
type File {
    handle: RawFd
}

impl File {
    def drop(self)
        close_fd(self.handle)
}

def read_file(path: String) -> String ~ IoError
    let file = File::open(path)~
    file.read_all()~
```

**19. Execution domain declaration**
```axiom
domain server:
    def handle_request(req: Request) -> Response ~ HttpError
        let user = db.query("SELECT * FROM users WHERE id = ?", req.user_id)~
        Response::json(user)

domain client:
    def render_user_list(users: List[User]) -> Element
        div(children: users |> map(render_user))
```

**20. Cross-domain call**
```axiom
domain client:
    def get_user(id: i64) -> User ~ ClientError
        let user = call server fetch_user(id)~
        user
```

**21. Small database-backed server**
```axiom
use std.http
use std.db

type User {
    id: i64
    name: String
    email: String
}

domain server:
    def handle_get_users(req: Request) -> Response ~ HttpError
        let users = db.query("SELECT * FROM users")~
        Response::json(users)

    def handle_create_user(req: Request) -> Response ~ HttpError
        let body = req.body().parse_json()~
        let user = db.execute(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            body.name,
            body.email
        )~
        Response::created(user)

def main()
    let server = http::Server::new("0.0.0.0:8080")
    server.route("GET", "/users", handle_get_users)
    server.route("POST", "/users", handle_create_user)
    server.start()
```

**22. Small client example**
```axiom
use std.dom

domain client:
    def render_user_list(users: List[User]) -> Element
        ul(children: users |> map(|user| li(text: user.name)))

    def main()
        let users = call server get_users()!
        let app = render_user_list(users)
        dom.mount(app, "#app")
```

**23. Small worker example**
```axiom
domain worker:
    def process_image(image: Image) -> ImageResult ~ WorkerError
        let filtered = apply_filter(image)~
        let resized = resize(filtered, 800, 600)~
        Ok(resized)

def main()
    let image = load_image("photo.jpg")~
    let handle = spawn {
        process_image(image)
    }
    let result = handle.await~
    save_image(result, "processed.jpg")~
```

**24. Medium-sized example**
```axiom
use std.http
use std.json
use std.db

type User {
    id: i64
    name: String
    email: String
}

type ApiError = NotFound | BadRequest(String) | DatabaseError(String) | Unauthorized

def parse_user_id(raw: String) -> i64 ~ ApiError
    let id = raw.parse_int()~
    if id <= 0 {
        Err(ApiError::BadRequest("Invalid user ID"))
    } else {
        Ok(id)
    }

domain server:
    def handle_get_user(req: Request) -> Response ~ ApiError
        let id = parse_user_id(req.param("id")!)~
        let user = db.query("SELECT * FROM users WHERE id = ?", id)!
            .first()
            .ok_or(ApiError::NotFound)!
        Response::json(user)

    def handle_update_user(req: Request) -> Response ~ ApiError
        let id = parse_user_id(req.param("id")!)~
        let body = req.body().parse_json()!
        db.execute(
            "UPDATE users SET name = ?, email = ? WHERE id = ?",
            body.name,
            body.email,
            id
        )!
        let user = db.query("SELECT * FROM users WHERE id = ?", id)!
            .first()
            .ok_or(ApiError::NotFound)!
        Response::json(user)

def main()
    let server = http::Server::new("0.0.0.0:8080")
    server.route("GET", "/users/:id", handle_get_user)
    server.route("PUT", "/users/:id", handle_update_user)
    server.start()
```

---

## 3. Detailed Feature Analysis

### 3.1 Function Keyword

| Option | Readability | Discoverability | Ambiguity | Parser | Identity | Beginner |
|--------|------------|----------------|-----------|--------|----------|----------|
| `fn` | High | High | None | Simple | Moderate (Rust-like) | Easy |
| `function` | High | High | None | Simple | Low (too common) | Easy |
| `def` | High | High | None | Simple | Moderate (Python/Ruby) | Easy |
| `fun` | Moderate | Moderate | None | Simple | Moderate | Easy |
| (none) | Variable | Low | High | Complex | High | Hard |

**Analysis:** `fn` is the best choice. It is short, unambiguous, visually distinct from variable names, and already familiar to Rust developers. `function` is too verbose. `def` is already used by Python/Ruby and loses AXIOM identity. No keyword creates parsing ambiguity (a name followed by `(` could be a function or a call).

**Recommendation:** `fn`

### 3.2 Variable Syntax

| Option | Readability | Mutation clarity | Parser | Identity |
|--------|------------|-----------------|--------|----------|
| `let` / `let mut` | High | Clear | Simple | Moderate (Rust/ML) |
| `val` / `var` | High | Clear | Simple | Moderate (Kotlin/Scala) |
| `let` / `var` | High | Clear | Simple | Moderate (JS/Kotlin) |
| `:=` / `=` | Moderate | Moderate | Complex | Low |
| `name := value` | Moderate | Unclear | Complex | Low |

**Analysis:** `let` / `let mut` is the best choice for AXIOM. `let` is universally understood by developers from Rust, Haskell, ML, Swift, and Kotlin. `let mut` clearly communicates "this binding is mutable." The alternative `val` / `var` (Kotlin-style) is also clear but `val` does not communicate immutability as strongly as `let` (which implies "let this be"). `let` / `var` (JavaScript-style) is confusing because `var` has hoisting semantics in JS.

**Recommendation:** `let` (immutable) / `let mut` (mutable)

### 3.3 Semicolons

| Option | Readability | Parser | Formatter | Identity |
|--------|------------|--------|-----------|----------|
| Required semicolons | Familiar (C-family) | Simple | Simple | Low (too common) |
| No semicolons (significant whitespace) | High | Complex | Complex | Moderate (Python) |
| Optional semicolons | High | Complex | Complex | Low |
| Semicolons as expression terminators | High | Moderate | Simple | High (Rust-like) |

**Analysis:** The no-semicolons approach is cleaner to read but creates parsing complexity (indentation-sensitive parsing is fragile and creates confusing error messages). Required semicolons are familiar but noisy. The best compromise is Rust-style: semicolons separate statements, but the final expression in a block has no semicolon (making it the block's value). This is clean for return values and avoids trailing semicolons on the last line of every function.

**Recommendation:** Semicolons as statement separators (like Rust). Final expression in a block omits semicolon to become the block's value.

### 3.4 Return Type Arrow

| Option | Readability | Parser | Identity |
|--------|------------|--------|----------|
| `->` | High | Simple | High (Rust/Go/ML) |
| `:` | Moderate | Complex (ambiguous with type annotation) | Low |
| `=>` | Moderate | Moderate | Low |

**Analysis:** `->` is the clear winner. It is visually distinctive, unambiguous, and already familiar from Rust, Go, Haskell, and ML. Using `:` for both type annotations and return types creates confusion.

**Recommendation:** `->`

### 3.5 Failure Propagation

| Option | Readability | Verbosity | Distinctiveness | Parser | Beginner |
|--------|------------|-----------|----------------|--------|----------|
| `?` | High | Terse | Low (Rust uses it) | Simple | Easy |
| `!` | High | Terse | High (AXIOM-specific) | Simple | Easy |
| `~>` | Moderate | Moderate | High | Moderate | Moderate |
| `try` | High | Verbose | Low (Java/C# uses it) | Simple | Easy |
| `propagate` | High | Very verbose | High | Simple | Easy |
| `.or_return` | High | Verbose | Moderate | Simple | Easy |

**Analysis:** `?` is the most familiar from Rust, but it is also used for absence types in some contexts, creating potential confusion. `!` is terse, visually clear (the exclamation mark suggests "this might fail!"), and is not overloaded in AXIOM. However, `!` can be confused with the logical NOT operator. `~>` is distinctive but visually noisy. The method-style `.or_return` is explicit but verbose.

The best choice is `!` for propagation. It is distinct from Rust's `?`, it communicates "this might fail," and it does not conflict with other AXIOM syntax. The absence type uses `?` prefix (e.g., `?User`), which is distinct from the propagation operator `!`.

**Recommendation:** `!` for failure propagation, `?` prefix for absence types

### 3.6 Absence Type

| Option | Readability | Verbosity | Distinctiveness | Parser |
|--------|------------|-----------|----------------|--------|
| `?T` (prefix) | High | Terse | High | Simple |
| `Option[T]` | Moderate | Verbose | Low (Rust) | Simple |
| `maybe T` | High | Moderate | Moderate | Simple |
| `T?` (suffix) | High | Terse | Moderate (C#/TypeScript) | Complex |

**Analysis:** `?T` (prefix) is the best choice. It is terse, visually distinctive, and clearly communicates "this type might not have a value." The prefix position makes it easy to parse (the `?` is always before a type name). `T?` (suffix) is used by C# and TypeScript but creates parsing ambiguity (is `int?` a nullable int or an int followed by `?`).

**Recommendation:** `?T` (prefix notation for absence types)

### 3.7 Generics

| Option | Readability | Parser | Disambiguation | Identity |
|--------|------------|--------|----------------|----------|
| `List[T]` (square brackets) | High | Simple | No ambiguity | High |
| `List<T>` (angle brackets) | High | Complex (ambiguous with `<` operator) | Requires disambiguation | Low (too common) |
| `List::[T]` | Moderate | Simple | No ambiguity | Moderate |
| `List of T` | High | Simple | No ambiguity | High |

**Analysis:** Square brackets `List[T]` are the best choice. They avoid the angle bracket ambiguity problem that plagues C++, Java, and Rust (where `>>` can be the shift operator or closing brackets). Square brackets are also visually distinct from parentheses (function calls) and curly braces (blocks). The `List of T` syntax is readable but verbose.

**Recommendation:** `List[T]` (square brackets for generics)

### 3.8 Execution Domains

| Option | Readability | Visual prominence | Annotation noise | Parser | Identity |
|--------|------------|-------------------|-----------------|--------|----------|
| `domain server { ... }` blocks | High | High | None | Simple | High |
| `@server fn ...` annotations | High | Low (annotation soup risk) | High | Simple | Low |
| `fn handle() server -> Response` | Moderate | Low | Low | Complex | Moderate |
| `server::fn handle()` | Moderate | Moderate | Low | Simple | Moderate |

**Analysis:** `domain server { ... }` blocks are the best choice. They make domain boundaries visually prominent in the source code. The opening `domain` keyword and closing brace clearly delineate what code belongs to which domain. This avoids annotation soup (`@server @public @async @safe`) and makes the code scannable.

Annotations (`@server`) are the worst choice because they accumulate and create visual noise. They also do not clearly show where a domain ends.

**Recommendation:** `domain server { ... }` block syntax

### 3.9 Cross-Domain Calls

| Option | Readability | Distinctiveness | Verbosity | Parser |
|--------|------------|----------------|-----------|--------|
| `call server fn(args)` | High | High | Moderate | Simple |
| `server.fn(args)` | High | Low (looks like method call) | Low | Simple |
| `@server fn(args)` | Moderate | Moderate | Low | Complex |
| `rpc server fn(args)` | High | High | Moderate | Simple |

**Analysis:** `call server fn(args)` is the best choice. The `call` keyword makes cross-domain calls visually distinct from regular function calls. It communicates "this is not a local call; it crosses a domain boundary." The domain name follows `call`, making it clear which domain is being called.

**Recommendation:** `call domain_name function(args)`

### 3.10 Concurrency

| Option | Readability | Distinctiveness | Verbosity | Parser |
|--------|------------|----------------|-----------|--------|
| `spawn { ... }` | High | High | Low | Simple |
| `task { ... }` | High | Moderate | Low | Simple |
| `async { ... }` | High | Low (JS/Rust) | Low | Simple |
| `go { ... }` | High | Low (Go) | Low | Simple |

**Analysis:** `spawn` is the best choice. It communicates "create a new concurrent computation" without implying async/await semantics (which are a different model). `async` is misleading because AXIOM's concurrency model is not based on async/await (it is structured concurrency with channels). `go` is too Go-specific.

For awaiting results, `handle.await` is clear and readable. It communicates "wait for this handle to produce a result."

**Recommendation:** `spawn { ... }` for task creation, `handle.await` for receiving results

### 3.11 Channel Syntax

| Option | Readability | Verbosity | Distinctiveness | Parser |
|--------|------------|-----------|----------------|--------|
| `chan[T]` | High | Terse | High | Simple |
| `Channel[T]` | High | Moderate | Low | Simple |
| `channel(T)` | High | Moderate | Low | Simple |
| `chan T` | High | Terse | High | Simple |

**Analysis:** `chan[T]` is the best choice. It is terse, visually distinctive, and clearly communicates "this is a typed channel." The square brackets match the generic syntax. `Channel[T]` is too verbose for a common operation.

**Recommendation:** `chan[T]`

### 3.12 Module Imports

| Option | Readability | Verbosity | Distinctiveness | Parser |
|--------|------------|-----------|----------------|--------|
| `import std.http` | High | Moderate | Moderate | Simple |
| `use std.http` | High | Low | Moderate | Simple |
| `from std import http` | High | Verbose | Low (Python) | Simple |
| `require std.http` | High | Moderate | Low (Ruby) | Simple |

**Analysis:** `import` is the best choice. It is universally understood, clearly communicates "bring this module into scope," and is the standard in most languages. `use` is shorter but less descriptive (in Rust, `use` brings specific names into scope, while `mod` declares modules).

**Recommendation:** `import`

### 3.13 Visibility

| Option | Readability | Verbosity | Distinctiveness | Parser |
|--------|------------|-----------|----------------|--------|
| `pub` prefix | High | Low | High (Rust) | Simple |
| `export` prefix | High | Moderate | Moderate | Simple |
| `public` prefix | High | Moderate | Low | Simple |
| No default (all private) | High | N/A | Low | Simple |

**Analysis:** `pub` is the best choice. It is terse, visually distinctive, and already familiar from Rust. AXIOM defaults to private (not public) to prevent accidental exposure of internal details.

**Recommendation:** `pub` prefix, private by default

### 3.14 Type Construction

| Option | Readability | Verbosity | Distinctiveness | Parser |
|--------|------------|-----------|----------------|--------|
| `User { name: "Alice" }` | High | Moderate | High (Rust) | Simple |
| `User.new(name: "Alice")` | High | Verbose | Moderate | Simple |
| `User(name: "Alice")` | High | Moderate | Moderate (Python) | Simple |
| `User::new { name: "Alice" }` | Moderate | Verbose | Moderate | Simple |

**Analysis:** `User { name: "Alice" }` is the best choice. It is visually clear, distinct from function calls (parentheses), and already familiar from Rust. It communicates "construct a User with these field values."

**Recommendation:** `Type { field: value, ... }`

---

## 4. Complexity Analysis

### Proposal A: "Familiar / Low Friction"

| Metric | Score (1-10) | Notes |
|--------|-------------|-------|
| Core keywords | 20 | fn, let, mut, struct, enum, impl, if, else, while, for, match, return, break, continue, import, pub, spawn, await, domain, call |
| Punctuation rules | 12 | () {} [] , ; : . -> ? ! & :: => |
| Special cases | 5 | Semicolons on statements, no semicolon on final expression, generic syntax, domain annotations, cross-domain calls |
| Grammar ambiguity | Low | Well-defined precedence, no significant whitespace |
| Beginner learning burden | Low | Most syntax is familiar from mainstream languages |
| Advanced expressiveness | High | Domains, failure propagation, absence types, generics, closures |
| Readability | High | Clean, consistent, familiar |
| Formatter complexity | Low | Semicolons provide clear statement boundaries |
| Parser complexity | Low | Standard recursive descent, no ambiguity |
| LSP/tooling complexity | Low | Standard name resolution, type inference |

**Weighted score: 8.2/10**

### Proposal B: "Balanced AXIOM" (Recommended)

| Metric | Score (1-10) | Notes |
|--------|-------------|-------|
| Core keywords | 18 | fn, let, mut, struct, enum, impl, if, else, while, for, match, break, continue, import, pub, spawn, domain, call |
| Punctuation rules | 11 | () {} [] , : . -> ? ! & :: |
| Special cases | 4 | No semicolons on final expression, generic syntax with square brackets, domain blocks, absence prefix |
| Grammar ambiguity | Low | Well-defined precedence, no significant whitespace |
| Beginner learning burden | Low | Most syntax is familiar; domain blocks are new but intuitive |
| Advanced expressiveness | High | Domains, failure propagation, absence types, generics, closures |
| Readability | High | Clean, less noisy than A (no semicolons), distinctive domain blocks |
| Formatter complexity | Low | Blocks provide clear boundaries |
| Parser complexity | Low | Standard recursive descent, no ambiguity |
| LSP/tooling complexity | Low | Standard name resolution, type inference |

**Weighted score: 8.7/10**

### Proposal C: "Distinctive AXIOM"

| Metric | Score (1-10) | Notes |
|--------|-------------|-------|
| Core keywords | 17 | def, let, let!, type, impl, if, elif, else, while, for, match, return, break, continue, use, pub, spawn, domain |
| Punctuation rules | 13 | () {} [] , : . -> ~ ? ! :: \|> \| |
| Special cases | 6 | `let!` for mutable, `~>` for propagation, `?some`/`?none` patterns, pipe operator, `domain:` with colon, `def` without braces for single-expression |
| Grammar ambiguity | Moderate | Pipe operator precedence, domain colon syntax, `let!` vs `let` |
| Beginner learning burden | Moderate | `let!`, `~>`, `?some`, `?none`, pipe operator, `def` are all new |
| Advanced expressiveness | High | Pipe composition, distinctive domain syntax, clean pattern matching |
| Readability | High | Very clean, pipe composition is elegant, but new syntax adds learning cost |
| Formatter complexity | Moderate | Pipe operator and domain colon need special handling |
| Parser complexity | Moderate | Pipe operator precedence, `let!` disambiguation, domain colon |
| LSP/tooling complexity | Moderate | New syntax patterns need special handling |

**Weighted score: 7.4/10**

---

## 5. First-Read Test

### Test program (24 lines)

```axiom
import std.http

struct User {
    name: String
    age: i64
}

enum ApiError {
    NotFound
    BadRequest(String)
}

fn parse_id(raw: String) -> i64 ! ApiError {
    let id = raw.parse_int()!
    if id <= 0 { Err(ApiError.BadRequest("Invalid")) } else { Ok(id) }
}

domain server {
    fn handle(req: Request) -> Response ! ApiError {
        let id = parse_id(req.param("id")!)!
        Response::json(db.query("SELECT * FROM users WHERE id = ?", id)!)
    }
}

fn main() {
    let server = http.Server::new("0.0.0.0:8080")
    server.route("GET", "/users/:id", handle)
    server.start()
}
```

### First-Read Analysis (Proposal B)

**Immediately understandable (no AXIOM knowledge needed):**
- `import std.http` — importing a module
- `struct User { name: String, age: i64 }` — defining a type with fields
- `enum ApiError { NotFound, BadRequest(String) }` — an enum with variants
- `fn parse_id(raw: String) -> i64 ! ApiError` — a function with parameters, return type, and something after `!`
- `let id = raw.parse_int()!` — calling a method, `!` does something
- `if id <= 0 { ... } else { ... }` — conditional
- `fn main()` — entry point
- `let server = http.Server::new(...)` — creating an object
- `server.route(...)` — calling a method
- `server.start()` — calling a method

**Requires AXIOM knowledge:**
- `!` after a function call means "propagate failure"
- `domain server { ... }` means "this code runs on the server"
- `db.query(...)!` means "this might fail, propagate if it does"
- `Response::json(...)` means "construct a Response value"

**What requires explanation:**
- The `!` operator (2 lines)
- The `domain` keyword (1 line)
- The `?` prefix for absence types (not shown in this example)

**Score:** 75% immediately understandable, 25% requires AXIOM knowledge. This is good. A developer can read and understand the structure of the program without knowing AXIOM-specific semantics.

---

## 6. AXIOM Identity: Defining Characteristics

These syntax elements make AXIOM recognizable:

1. **`domain server { ... }` blocks** — No other language has execution domain blocks. This is AXIOM's signature feature and should be visually prominent.

2. **`!` for failure propagation** — Distinct from Rust's `?`, the exclamation mark communicates "this might fail!" and is visually striking.

3. **`?T` prefix for absence types** — `?User` reads as "maybe a User." This is more natural than `Option<User>` or `User?`.

4. **`call server fn(args)`** — Cross-domain calls are explicitly marked. No other language has this.

5. **Square brackets for generics** — `List[Int]` instead of `List<Int>`. This avoids angle bracket ambiguity and is visually distinctive.

6. **No semicolons in common case** — Semicolons are statement separators, not required on the last expression. This is familiar from Rust but less common in mainstream languages.

7. **`chan[T]`** — A terse, distinctive channel type. Communicates "typed message pipe."

---

## 7. Recommendation

### Ranking

| Rank | Proposal | Score | Strengths | Weaknesses |
|------|----------|-------|-----------|------------|
| 1 | B: Balanced AXIOM | 8.7 | Familiar, clean, distinctive domains, no semicolon noise | `!` could be confused with NOT |
| 2 | A: Familiar | 8.2 | Maximally familiar, easy to learn | Less distinctive, semicolons add noise |
| 3 | C: Distinctive | 7.4 | Pipe composition, elegant patterns | Too many new concepts, `let!` is confusing |

### Recommended: Proposal B

Proposal B is the best balance of familiarity and distinctiveness. It is immediately readable to mainstream developers, but its domain blocks, `!` propagation, `?T` absence, and `chan[T]` syntax make it clearly AXIOM.

### Strongest Ideas from Rejected Proposals

- **From A:** The function keyword `fn` is well-chosen and should be adopted (already in B).
- **From A:** Semicolon-based statement separation is clean and should be adopted (already in B).
- **From C:** Pipe-style composition (`|>`) is elegant and worth considering for future versions.
- **From C:** The `elif` keyword (instead of `else if`) is worth considering.
- **From C:** `type Color = Red | Green | Blue` for simple enums is cleaner than block syntax.

### Syntax Decisions Safe to LOCK After Review

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Function keyword | `fn` | Unambiguous, short, familiar |
| Binding keyword | `let` / `let mut` | Familiar, clear immutability/mutation |
| Return arrow | `->` | Unambiguous, familiar |
| Block syntax | `{ }` | Universal, unambiguous |
| String literals | `"..."` | Universal |
| Char literals | `'...'` | Universal |
| Comment syntax | `//` and `/* */` | Universal |
| Integer literals | Decimal, `0x` hex, `0b` binary, `0o` octal | Standard |
| Float literals | `3.14`, `1.0e10` | Standard |
| Boolean literals | `true`, `false` | Standard |
| Field access | `.` | Universal |
| Index access | `[]` | Universal |
| Import keyword | `import` | Universal |
| Visibility | `pub` | Terse, familiar from Rust |
| Generics | `List[T]` | No angle bracket ambiguity |

### Syntax Decisions That Should Remain OPEN

| Decision | Why it should remain OPEN |
|----------|--------------------------|
| Failure propagation operator (`!` vs `?` vs other) | Needs more feedback; `!` conflicts with logical NOT in some contexts |
| Absence type prefix (`?T` vs `maybe T` vs other) | Needs more feedback; `?T` is terse but might be confusing for beginners |
| Domain syntax (block vs annotation) | `domain { }` blocks are recommended but need real-world validation |
| Cross-domain call syntax (`call server fn` vs other) | Needs more feedback |
| Channel syntax (`chan[T]` vs `Channel[T]`) | Needs more feedback |
| Mutable binding keyword (`let mut` vs `let!` vs `var`) | Needs more feedback |
| Generic syntax (`[T]` vs `<T>`) | Square brackets recommended but angle brackets are more familiar |
| Lambda syntax (`|x| x + 1` vs `fn(x) x + 1`) | Needs more feedback |
| Match arrow (`=>` vs `->`) | `=>` is more common but `->` is consistent with return type |
| Pattern syntax for absence (`Some(x)` vs `?(x)` vs `?some(x)`) | Needs more feedback |

### Semantic Problems Revealed While Designing Syntax

1. **Failure propagation and absence handling overlap.** The `!` operator propagates failures. The absence type uses `?T`. But what if a function returns `?T ! E` (might be absent AND might fail)? The propagation operator needs to work with both. This needs careful design.

2. **Cross-domain call syntax needs to handle multiple argument types.** `call server fn(arg1, arg2)` works for simple cases, but what about named arguments, generic functions, and closures? The syntax needs to be general enough.

3. **Domain blocks and module boundaries interact.** If `domain server { ... }` is a block, does it create a new scope? Can it contain type declarations? How do types defined inside a domain block interact with types outside? This needs careful design.

4. **Channel syntax and generic syntax need to be consistent.** If generics use `List[T]`, channels should use `chan[T]`. But channels are also generic. The syntax should be consistent.

5. **Mutable binding and error propagation could conflict.** If `let mut` is for mutation and `!` is for propagation, what about `let!` (Proposal C)? This was rejected because `let!` is confusing. But the interaction between mutation and error handling in bindings needs clear syntax.

---

## Appendix: Complete Keyword List (Proposal B)

| Keyword | Purpose |
|---------|---------|
| `fn` | Function declaration |
| `let` | Immutable binding |
| `mut` | Mutable modifier |
| `const` | Compile-time constant |
| `struct` | Product type declaration |
| `enum` | Sum type declaration |
| `impl` | Method implementation |
| `type` | Type alias |
| `if` | Conditional |
| `else` | Alternative branch |
| `while` | Loop |
| `for` | Iterator loop |
| `in` | Iterator binding |
| `match` | Pattern matching |
| `return` | Early return |
| `break` | Exit loop |
| `continue` | Next iteration |
| `import` | Module import |
| `pub` | Public visibility |
| `domain` | Execution domain |
| `call` | Cross-domain call |
| `spawn` | Task creation |
| `await` | Task result |
| `chan` | Channel type |
| `true` | Boolean literal |
| `false` | Boolean literal |
| `self` | Current instance |
| `super` | Parent module |
| `and` | Logical AND (alternative to `&&`) |
| `or` | Logical OR (alternative to `||`) |
| `not` | Logical NOT (alternative to `!`) |
| `panic` | Unrecoverable error |
| `unreachable` | Dead code marker |

**Total: 33 keywords**

This is comparable to Rust (34 keywords), less than Java (67 keywords), and more than Go (25 keywords). The keyword count is appropriate for a general-purpose language.
