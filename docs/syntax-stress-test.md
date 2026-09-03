# AXIOM Syntax Stress Test — Phase 2.5

## Executive Summary

Proposal B is structurally sound but has **3 critical problems**, **5 major problems**, and **8 minor problems** that must be addressed before grammar design. The most serious issue is the **`!` propagation operator conflicting with logical NOT**, which creates genuine parsing ambiguity and semantic confusion. The second critical problem is that **`?T` absence prefix and `!` propagation are visually unrelated concepts that use similar punctuation**, creating confusion about when to use which. The third is that **`call server fn(args)` breaks when the target has generics or complex types**.

**PROPOSAL B STATUS: APPROVE WITH CHANGES**

The core structure is sound. The following changes are required before grammar design:

1. Replace `!` propagation with `?` (adopt Rust's convention — it is less confusing)
2. Rename absence prefix from `?T` to `maybe T` (or `opt T`) to avoid collision with propagation
3. Redesign cross-domain call syntax to handle generics and complex types
4. Add explicit ownership/borrowing syntax (currently missing from Proposal B)
5. Clarify channel creation syntax (currently ambiguous between type and constructor)

These changes are small, targeted, and do not require redesigning the language.

---

## 1. Critical Problems

### CRITICAL-1: `!` Propagation Conflicts with Logical NOT

**CURRENT:**
```axiom
if !done { ... }
let value = expr()!
```

**PROBLEM:**
`!` is used for both logical NOT and failure propagation. The parser cannot distinguish `!expr` (NOT) from `expr!` (propagation) without looking at context. While `!expr` (prefix) and `expr!` (postfix) are syntactically different positions, this creates:

1. **Visual confusion** — `!done` vs `done!` look similar but mean completely different things
2. **Beginner confusion** — "Does `!` mean NOT or propagation?"
3. **Future extensibility** — If AXIOM ever adds a prefix `!` operator (e.g., for bitwise NOT or compile-time assertion), it will collide with propagation
4. **Syntax highlighting** — Editors must distinguish `!` by position, which is fragile

**RECOMMENDED:**
Replace `!` propagation with `?` (Rust's convention).

**REASON:**
`?` is postfix-only (`expr?`), has no prefix meaning in AXIOM, and is already familiar to developers from Rust. The downside is that `?` is used for absence types (`?User`), but these are different positions: `?T` is a type prefix, `expr?` is an expression postfix. The parser can distinguish them.

**ALTERNATIVE:**
Keep `!` for propagation but rename absence from `?T` to `maybe T`. This avoids the collision but introduces a new keyword.

---

### CRITICAL-2: `?T` Absence and `!` Propagation Use Similar Punctuation

**CURRENT:**
```axiom
fn find_user(id: i64) -> ?User { ... }
fn load(id: i64) -> User ! DatabaseError { ... }
let user = find_user(1)!  // absence unwrap or failure propagate?
```

**PROBLEM:**
When a function returns `?User ! DatabaseError` (absent OR failed), the programmer must use both `?` and `!` on the same expression. The syntax becomes:

```axiom
let user = find_user(1)!?  // propagate failure, then unwrap absence?
let user = find_user(1)?!  // unwrap absence, then propagate failure?
```

The order matters but is not intuitive. The programmer must remember which comes first. This is a real source of bugs.

**RECOMMENDED:**
If propagation becomes `?`, then absence should NOT use `?` prefix. Use `maybe T` instead.

**REASON:**
`maybe T` is a keyword, not punctuation. It is visually distinct from `?` propagation. The expression `find_user(1)?` is clean (propagate failure). To handle absence, use methods: `find_user(1)?.or(default)`.

---

### CRITICAL-3: Cross-Domain Call Syntax Breaks with Generics

**CURRENT:**
```axiom
call server fetch_user(id)
call server db.query("SELECT * FROM users")
```

**PROBLEM:**
The `call domain function(args)` syntax assumes the function is a simple name. But what about:

```axiom
call server db.query[Row]("SELECT * FROM users")  // generic function
call server fetch_user(id)  // which overload?
call server process(data, config={verbose: true})  // named args
```

The `call` keyword makes these awkward. The syntax `call server fn(args)` looks like a command, not a function call. It does not compose well with method calls, generic functions, or named arguments.

**RECOMMENDED:**
Use domain-qualified function calls instead of `call`:

```axiom
server::fetch_user(id)
server::db.query("SELECT * FROM users")
```

**REASON:**
`::` is the namespace operator (already used for type construction: `User::new`). Using it for domain qualification is consistent. `server::fetch_user(id)` reads naturally as "the fetch_user function in the server domain." It composes with generics (`server::fn[T](args)`), method calls (`server::obj.method()`), and named arguments.

---

## 2. Major Problems

### MAJOR-1: No Explicit Ownership/Borrowing Syntax in Proposal B

**CURRENT:**
Proposal B does not specify syntax for borrowing. The language-core-spec says borrowing is a semantic requirement, but Proposal B provides no notation.

**PROBLEM:**
Without borrowing syntax, the programmer cannot:
- Pass a value by reference instead of by value
- Create a shared reference for reading
- Create an exclusive reference for mutation
- Express lifetime constraints

This means Proposal B is incomplete for the native target (which uses reference counting and needs explicit borrows) and for concurrency (which needs to prevent data races).

**RECOMMENDED:**
Add `&T` for shared references and `&mut T` for exclusive references. This is minimal, unambiguous, and already familiar from Rust.

**REASON:**
The `&` prefix is already used in many languages. It does not collide with any existing AXIOM syntax. The `&mut` combination is clear and explicit. The borrow checker can enforce these at compile time.

---

### MAJOR-2: Channel Creation Syntax Is Ambiguous

**CURRENT:**
```axiom
let ch = chan[Int]
```

**PROBLEM:**
Is `chan[Int]` a type or a value? In `let ch = chan[Int]`, the right side should be a value, but `chan[Int]` looks like a type annotation. The programmer might expect:

```axiom
let ch: chan[Int] = chan::new()  // explicit construction
let ch = chan::new[Int]()  // generic constructor
let ch = chan[Int]()  // construction with type parameter
```

The current syntax is ambiguous between type and value.

**RECOMMENDED:**
Use `chan::new[T]()` for construction:

```axiom
let ch = chan::new[Int]()
```

**REASON:**
This is consistent with `User::new { ... }` (type construction with `::new`). The `[T]` is a type parameter on the constructor. The `()` indicates construction. This is unambiguous.

---

### MAJOR-3: `fn` Closure Syntax Ambiguous with Function Type

**CURRENT:**
```axiom
fn add(a: Int, b: Int) -> Int { a + b }  // function declaration
let f = fn(x) x + 1  // closure
let g: fn(Int) -> Int = fn(x) x + 1  // closure with type annotation
```

**PROBLEM:**
`fn` is used for both function declarations and closures. The parser must distinguish:

- `fn name(params) -> Type { body }` — declaration
- `fn(params) -> Type { body }` — anonymous function
- `fn(params) expr` — shorthand closure
- `fn(x) expr` — single-parameter shorthand

The shorthand `fn(x) expr` is ambiguous: is `fn(x)` a function call on `fn` with argument `x`, or a closure with parameter `x`?

**RECOMMENDED:**
Use `|x| expr` for closures (like Rust, Kotlin, Swift):

```axiom
let f = |x| x + 1
let g: fn(Int) -> Int = |x| x + 1
list.map(|x| x * 2)
```

**REASON:**
`|x|` is unambiguous: the pipe characters clearly delimit parameters. It does not conflict with `fn` (which is only for declarations). It is familiar from Rust, Kotlin, and Swift. The `fn(Int) -> Int` type annotation remains for function types.

---

### MAJOR-4: Match Arrow `=>` vs Return Arrow `->`

**CURRENT:**
```axiom
match color {
    Red => "stop"
    Green => "go"
}

fn add(a: Int) -> Int { a + 1 }
```

**PROBLEM:**
`=>` is used for match arms, `->` is used for return types. These are visually similar but semantically different. A beginner might confuse them:

```axiom
fn grade(score: Int) -> String {  // return type
    match score {
        s if s >= 90 => "A",  // match arm
        ...
    }
}
```

The visual similarity between `->` and `=>` is a source of confusion.

**RECOMMENDED:**
Keep both `->` and `=>`. They are in different contexts (type position vs expression position) and the parser can distinguish them. Document the difference clearly.

**REASON:**
Changing either would break consistency with existing conventions. `->` is universally used for return types. `=>` is universally used for match arms. The visual similarity is a minor issue that is resolved by context. The parser has no ambiguity because `->` appears in type position and `=>` appears in expression position.

---

### MAJOR-5: Semicolon Omission Rules Are Unclear

**CURRENT:**
Proposal B says "final expression may omit semicolon." But what exactly counts as a "final expression"?

**PROBLEM:**
Consider:

```axiom
fn foo() -> Int {
    let x = 5
    x + 1  // no semicolon — this is the return value
}

fn bar() -> Int {
    let x = 5;
    x + 1;  // semicolon — this returns ()
}
```

The difference between `x + 1` (return value) and `x + 1;` (discarded value) is a single semicolon. This is a common source of bugs in Rust (where the same rule exists). The error message is confusing: "expected Int, found ()".

**RECOMMENDED:**
Adopt Rust's rule but make the error message very clear:

- The last expression without a semicolon is the block's value
- The last expression with a semicolon is a statement (value is `()`)
- The compiler produces a clear error: "block returns (), expected Int — did you forget to remove the semicolon?"

**REASON:**
This rule is simple once learned. The error message must be excellent to compensate for the learning curve. This is a known issue from Rust and the error message design is the fix.

---

## 3. Minor Problems

### MINOR-1: `impl` Block Syntax Underspecified

**CURRENT:**
```axiom
impl User {
    fn new(name: String, age: Int) -> User { ... }
    fn drop(self) { ... }
}
```

**PROBLEM:**
How does `impl` interact with the type definition? Can `impl` be in a different module than the type? Can there be multiple `impl` blocks? Can `impl` add methods to types from other modules?

**RECOMMENDED:**
Document the rules:
- `impl` must be in the same module as the type definition
- Multiple `impl` blocks are allowed (for organization)
- `impl` cannot add methods to types from other modules (no extension methods)

---

### MINOR-2: `type` Alias Syntax Conflicts with `struct`/`enum`

**CURRENT:**
```axiom
type UserId = Int
struct User { id: UserId, name: String }
```

**PROBLEM:**
`type` is used for aliases, `struct` and `enum` for type definitions. The keyword `type` is overloaded: it means "alias" in `type UserId = Int` but "type" in general conversation. A beginner might try `type User { ... }` instead of `struct User { ... }`.

**RECOMMENDED:**
Keep `type` for aliases. Use `struct` and `enum` for definitions. Document clearly that `type` is for aliases only.

---

### MINOR-3: `import` Path Separator Inconsistency

**CURRENT:**
```axiom
import std.http
import users
```

**PROBLEM:**
`std.http` uses `.` as a path separator. But in type construction, `::` is used (`User::new`). Should imports use `.` or `::`?

**RECOMMENDED:**
Use `.` for module paths in imports (`import std.http`). Use `::` for type members (`User::new`). These are different contexts and the inconsistency is justified.

**REASON:**
`.` is the standard path separator in most languages (Python, JavaScript, Java). `::` is the standard namespace separator for type members (Rust, C++, Haskell). Using different separators for different purposes is common and unambiguous.

---

### MINOR-4: `await` Position

**CURRENT:**
```axiom
let result = handle.await
```

**PROBLEM:**
`.await` is a postfix operator. But what about awaiting multiple handles?

```axiom
let (a, b) = (handle1.await, handle2.await)  // sequential
let a = handle1.await  // how to await both concurrently?
```

The current syntax does not show how to await multiple tasks concurrently.

**RECOMMENDED:**
Add a `join` or `all` primitive for concurrent awaiting:

```axiom
let (a, b) = join(handle1, handle2)
```

**REASON:**
`join` is a common concurrency primitive. It communicates "wait for all of these to complete." This is clearer than trying to use `.await` on multiple handles.

---

### MINOR-5: `pub` Placement Inconsistency

**CURRENT:**
```axiom
pub struct User { ... }
pub fn public_function() { ... }
pub name: String  // field visibility
```

**PROBLEM:**
`pub` is a prefix on declarations. But for struct fields, it is inside the struct body. The placement is inconsistent.

**RECOMMENDED:**
Keep `pub` as a prefix on declarations. Inside struct bodies, `pub` is a field modifier. This is consistent with Rust and is clear in context.

---

### MINOR-6: `domain` Block Scope

**CURRENT:**
```axiom
domain server {
    fn handle(req: Request) -> Response { ... }
}
```

**PROBLEM:**
Does `domain server { ... }` create a new scope? Can names defined inside be accessed outside? Can names from outside be accessed inside?

**RECOMMENDED:**
`domain` blocks do NOT create a new scope for name resolution. They are purely for domain annotation. Names defined inside are accessible outside (subject to visibility rules). Names from outside are accessible inside.

**REASON:**
Domain blocks are not modules. They are domain annotations. Creating a new scope would break name resolution and force the programmer to re-import everything inside each domain block.

---

### MINOR-7: `match` Exhaustiveness Syntax

**CURRENT:**
```axiom
match color {
    Red => "stop"
    Green => "go"
    Blue => "sky"
}
```

**PROBLEM:**
How does the compiler know the match is exhaustive? By checking all variants? What if a new variant is added to `Color`?

**RECOMMENDED:**
The compiler checks exhaustiveness by verifying that all variants of the matched type are covered. If a new variant is added, existing matches become compile errors. This is the standard approach (Rust, Haskell, ML).

---

### MINOR-8: `for` Loop Destructuring

**CURRENT:**
```axiom
for item in list { ... }
```

**PROBLEM:**
What about iterating over maps (key-value pairs)?

```axiom
for (key, value) in map { ... }  // destructuring in for loop
```

**RECOMMENDED:**
Support destructuring in `for` loops:

```axiom
for (key, value) in map { ... }
for (i, item) in list.enumerate() { ... }
```

---

## 4. Stress Test Matrix

### A. Types + Generics

```axiom
// Basic generic type
let list: List[Int] = [1, 2, 3]

// Generic with user type
let users: List[User] = [user1, user2]

// Nested generics
let matrix: List[List[Int]] = [[1, 2], [3, 4]]

// Map type
let scores: Map[String, Int] = {"alice": 95, "bob": 87}

// Function type
let f: fn(Int) -> Int = |x| x + 1

// Generic function
fn first[T](items: List[T]) -> T { items[0] }

// Generic struct
struct Pair[A, B] { first: A, second: B }

// Generic enum
enum Result[T, E] { Ok(T), Err(E) }

// Nested generic function type
fn compose(f: fn(Int) -> Int, g: fn(Int) -> Int) -> fn(Int) -> Int {
    |x| f(g(x))
}
```

**Issues found:** None. Square bracket generics are clean and unambiguous.

### B. Absence

```axiom
// Basic absence
fn find_user(id: Int) -> ?User { ... }

// Absence in collection
let users: List[?User] = [find_user(1), find_user(2)]

// Nested absence (problematic)
let nested: ?List[User] = ...  // is this "maybe a List[User]" or "List of maybe User"?

// Absence + generic
let maybe_pair: ?Pair[Int, String] = ...

// Absence + function return
fn parse(s: String) -> ?Int { ... }
```

**Issues found:** `?List[User]` vs `List[?User]` is ambiguous in reading. The prefix `?` applies to the immediately following type. `?List[User]` means "maybe a List[User]". `List[?User]` means "List of maybe User". This is parseable but requires documentation.

### C. Failures

```axiom
// Basic failure
fn load(id: Int) -> User ! DatabaseError { ... }

// Failure + absence (problematic)
fn find(id: Int) -> ?User ! DatabaseError { ... }

// Failure + collection
fn load_all(ids: List[Int]) -> List[User] ! DatabaseError { ... }

// Nested failure
fn complex() -> Result[Int, AppError] ! IoError { ... }
```

**Issues found:** `?User ! DatabaseError` is visually noisy. The combination of absence and failure in one return type is rare but needs clean syntax.

### D. Failure Propagation

```axiom
// Basic propagation
fn process() -> Int ! AppError {
    let a = load(1)!  // propagate
    let b = load(2)!  // propagate
    Ok(a + b)
}

// Propagation through conditionals
fn validate(x: Int) -> Int ! ValidationError {
    if x < 0 { return Err(ValidationError.Negative) }
    let v = normalize(x)!  // propagate
    Ok(v)
}

// Propagation through loops
fn sum_all(ids: List[Int]) -> Int ! DatabaseError {
    let mut total = 0
    for id in ids {
        let value = load(id)!  // propagate inside loop
        total += value
    }
    Ok(total)
}

// Propagation through match
fn describe(r: Result[Int, String]) -> String ! AppError {
    match r {
        Ok(v) => Ok(format!("Got: {}", v)),
        Err(e) => Err(AppError.Wrapped(e))
    }
}

// Propagation through tasks
fn parallel_load(ids: List[Int]) -> List[User] ! DatabaseError {
    let handles: List[Handle[User ! DatabaseError]] = []
    for id in ids {
        handles.push(spawn { load(id) })
    }
    let mut users = []
    for h in handles {
        users.push(h.await!)  // propagate through task
    }
    Ok(users)
}

// Propagation across domains
domain client {
    fn get_user(id: Int) -> User ! ClientError {
        let user = server::fetch_user(id)!  // propagate
        Ok(user)
    }
}
```

**Issues found:** Propagation through tasks requires `h.await!` — the `!` after `.await` is visually awkward. Consider `h.await?` if propagation becomes `?`.

### E. Mutation

```axiom
// Basic mutation
let mut x = 5
x = 10

// Mutation + failure
fn process() -> Int ! Error {
    let mut total = 0
    for id in ids {
        total += load(id)!  // mutation + propagation
    }
    Ok(total)
}

// Mutation + absence
fn find_first(list: List[Int], target: Int) -> ?Int {
    for item in list {
        if item == target { return Some(item) }
    }
    None
}
```

**Issues found:** `let mut` is clear. No issues.

### F. Ownership / Borrowing

```axiom
// Ownership transfer (implicit)
fn consume(data: String) { ... }
let s = "hello".to_string()
consume(s)  // s is moved, can't use after this

// Shared borrow
fn print_user(user: &User) { ... }
print_user(&my_user)  // borrow, my_user still valid

// Exclusive borrow
fn update_user(user: &mut User) { ... }
update_user(&mut my_user)  // exclusive borrow

// Borrow + failure
fn process(data: &List[Int]) -> Int ! Error {
    let first = data.first()?  // borrow + propagate
    Ok(*first)
}

// Borrow in closures
let name = &user.name
let f = || print(name)  // borrows name

// Borrow across tasks (problematic)
spawn {
    print(&shared_data)  // borrow in task — is this safe?
}

// Borrow across domains (problematic)
domain client {
    fn get_name(user: &User) -> String {
        server::process_name(user)!  // borrow across domain?
    }
}
```

**Issues found:**
1. Proposal B does not include `&` and `&mut` syntax. This is a gap.
2. Borrowing across tasks needs careful design (shared references in concurrent code are dangerous).
3. Borrowing across domains is likely not allowed (domains are separate compilation units).

### G. Concurrency

```axiom
// Basic spawn
let handle = spawn { heavy_computation() }
let result = handle.await

// Spawn + failure
fn process() -> Int ! Error {
    let handle = spawn { load_data()! }
    let data = handle.await!  // propagate from task
    Ok(data)
}

// Channel creation
let ch = chan::new[Int]()
spawn { ch.send(42) }
let value = ch.receive()

// Channel + failure
let ch = chan::new[Int ! Error]()
spawn { ch.send(load(1)!) }
match ch.receive() {
    Ok(v) => print(v),
    Err(e) => print(e),
}

// Nested tasks
fn parallel_process() -> Int ! Error {
    let h1 = spawn { step1()! }
    let h2 = spawn { step2()! }
    let r1 = h1.await!
    let r2 = h2.await!
    Ok(r1 + r2)
}

// Task ownership
fn process(data: String) -> Int ! Error {
    let handle = spawn {
        process_data(data)  // data is moved into task
    }
    // can't use data here
    handle.await!
}
```

**Issues found:** `chan::new[Int]()` is verbose. Consider `chan[Int]` as a constructor shorthand.

### H. Modules

```axiom
// Import
import std.http
import std.json

// Public/private
pub struct User { pub name: String, age: Int }
pub fn public_api() { }
fn internal_helper() { }

// Module access
let user = users::find(1)
let response = http::get(url)
```

**Issues found:** `users::find(1)` — is `users` a module or a type? The `::` operator is overloaded. Need to clarify: `::` on a module name accesses module members, `::` on a type name accesses type members.

### I. Execution Domains

```axiom
// Server domain
domain server {
    fn handle_request(req: Request) -> Response ! HttpError {
        let user = db::query("SELECT * FROM users WHERE id = ?", req.user_id)!
        Response::json(user)
    }
}

// Client domain
domain client {
    fn render(users: List[User]) -> Element {
        ul(children: users.map(|u| li(text: u.name)))
    }
}

// Worker domain
domain worker {
    fn process(image: Image) -> ImageResult ! WorkerError {
        let filtered = apply_filter(image)!
        Ok(filtered)
    }
}

// Cross-domain call (with generic function)
domain client {
    fn get_users() -> List[User] ! ClientError {
        let users = server::fetch_users[List[User]]()!  // generic cross-domain
        Ok(users)
    }
}

// Cross-domain call (with named args)
domain client {
    fn search(query: String) -> List[Result] ! ClientError {
        let results = server::search(query: query, limit: 10)!  // named args
        Ok(results)
    }
}

// Cross-domain call (with failure)
domain client {
    fn get_user(id: Int) -> ?User ! ClientError {
        let user = server::fetch_user(id)!  // failure propagation
        Ok(Some(user))
    }
}

// Cross-domain call (with absence)
domain client {
    fn find_user(id: Int) -> ?User ! ClientError {
        let user = server::find_user(id)!  // absence + failure
        Ok(user)  // user is ?User, propagate failure
    }
}

// Shared types across domains
struct User { id: Int, name: String, email: String }

domain server {
    fn fetch_user(id: Int) -> ?User ! DatabaseError { ... }
}

domain client {
    fn display_user(user: User) -> Element { ... }
}

// Cross-domain channel
domain server {
    fn stream_updates(ch: Chan[Update]) {
        spawn {
            for update in changes {
                ch.send(update)
            }
        }
    }
}

domain client {
    fn receive_updates(ch: Chan[Update]) {
        spawn {
            for update in ch {
                render(update)
            }
        }
    }
}
```

**Issues found:**
1. `server::fetch_users[List[User]]()` — generic cross-domain calls are verbose
2. `server::search(query: query, limit: 10)` — named args across domains work but are verbose
3. `server::find_user(id)!` — failure + absence across domains is complex
4. Cross-domain channels — how are they created? Who owns them?

---

## 5. Realistic Application Test

```axiom
import std.http
import std.json
import std.db
import std.time

struct User {
    id: i64
    name: String
    email: String
    created_at: i64
}

struct Post {
    id: i64
    user_id: i64
    title: String
    body: String
    published: bool
}

enum ApiError {
    NotFound
    BadRequest(String)
    Unauthorized
    DatabaseError(String)
    SerializationError
}

fn validate_email(email: String) -> ?String {
    if email.contains("@") && email.contains(".") {
        Some(email)
    } else {
        None
    }
}

fn parse_id(raw: String) -> i64 ! ApiError {
    let id = raw.parse_i64()?
    if id <= 0 {
        Err(ApiError.BadRequest("Invalid ID".to_string()))
    } else {
        Ok(id)
    }
}

domain server {
    fn handle_get_user(req: Request) -> Response ! ApiError {
        let id = parse_id(req.param("id")?)?
        let user = db::query("SELECT * FROM users WHERE id = ?", id)?
            .first()?
            .ok_or(ApiError.NotFound)?
        Response::json(user)
    }

    fn handle_create_user(req: Request) -> Response ! ApiError {
        let body = req.body().parse_json::<CreateUserRequest>()?
        let email = validate_email(body.email)
            .ok_or(ApiError.BadRequest("Invalid email".to_string()))?
        db::execute(
            "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)",
            body.name, email, time::now()
        )?
        let user = db::query("SELECT * FROM users WHERE email = ?", email)?
            .first()?
            .ok_or(ApiError.NotFound)?
        Response::created(user)
    }

    fn handle_get_posts(req: Request) -> Response ! ApiError {
        let user_id = parse_id(req.param("user_id")?)?
        let posts = db::query("SELECT * FROM posts WHERE user_id = ? AND published = true", user_id)?
        Response::json(posts)
    }

    fn handle_create_post(req: Request) -> Response ! ApiError {
        let user_id = parse_id(req.param("user_id")?)?
        let body = req.body().parse_json::<CreatePostRequest>()?
        db::execute(
            "INSERT INTO posts (user_id, title, body, published) VALUES (?, ?, ?, ?)",
            user_id, body.title, body.body, false
        )?
        Response::created({"status": "draft"})
    }
}

domain client {
    fn render_user_profile(user: User, posts: List[Post]) -> Element {
        div(children: [
            h1(text: user.name),
            p(text: user.email),
            h2(text: "Posts"),
            ul(children: posts.map(|post| {
                li(children: [
                    h3(text: post.title),
                    p(text: post.body),
                ])
            }))
        ])
    }

    fn fetch_and_render(user_id: i64) -> Element ! ClientError {
        let user = call server handle_get_user(user_id)!
        let posts = call server handle_get_posts(user_id)!
        Ok(render_user_profile(user, posts))
    }
}

domain worker {
    fn send_welcome_email(user: User) -> ! EmailError {
        let template = load_template("welcome")?
        let body = template.render({"name": user.name})?
        email::send(user.email, "Welcome!", body)?
        Ok(())
    }

    fn process_image_batch(images: List[Image]) -> List[ImageResult] ! WorkerError {
        let handles = images.map(|img| spawn { process_single_image(img) })
        let results = handles.map(|h| h.await!)
        Ok(results)
    }
}

fn main() {
    let server = http::Server::new("0.0.0.0:8080")
    server.route("GET", "/users/:id", server::handle_get_user)
    server.route("POST", "/users", server::handle_create_user)
    server.route("GET", "/users/:user_id/posts", server::handle_get_posts)
    server.route("POST", "/users/:user_id/posts", server::handle_create_post)
    server.start()
}
```

**Issues found:**
1. `call server handle_get_user(user_id)!` — the `call` keyword + `!` propagation is verbose
2. `db::query(...)?.first()?.ok_or(ApiError.NotFound)?` — chain of `?` is hard to read
3. `validate_email(body.email).ok_or(ApiError.BadRequest("Invalid email".to_string()))?` — verbose error conversion
4. No explicit resource cleanup syntax shown
5. Cross-domain channel syntax not demonstrated

---

## 6. Beginner Readability Test

```axiom
import std.http

struct User {
    name: String
    age: i64
}

fn greet(user: User) -> String {
    format!("Hello, {}!", user.name)
}

fn is_adult(user: User) -> bool {
    user.age >= 18
}

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
        Some(u) => {
            print(greet(u))
            if is_adult(u) {
                print("Adult")
            }
        }
        None => print("User not found")
    }
}
```

**Classification:**

| Line | Classification | Notes |
|------|---------------|-------|
| `import std.http` | 1. Immediately understandable | Standard import |
| `struct User { ... }` | 1. Immediately understandable | Standard struct |
| `name: String` | 1. Immediately understandable | Field declaration |
| `fn greet(user: User) -> String` | 1. Immediately understandable | Function declaration |
| `format!("Hello, {}!", user.name)` | 2. Understandable from context | String formatting |
| `fn is_adult(user: User) -> bool` | 1. Immediately understandable | Boolean function |
| `user.age >= 18` | 1. Immediately understandable | Comparison |
| `fn find_user(id: i64) -> ?User` | 3. Requires AXIOM documentation | `?User` is new |
| `Some(User { name: "Alice", age: 30 })` | 3. Requires AXIOM documentation | `Some` constructor |
| `None` | 3. Requires AXIOM documentation | Absence literal |
| `match user { ... }` | 2. Understandable from context | Pattern matching |
| `Some(u) => { ... }` | 3. Requires AXIOM documentation | Pattern destructuring |
| `print(greet(u))` | 1. Immediately understandable | Function call |
| `if is_adult(u) { ... }` | 1. Immediately understandable | Conditional |
| `print("User not found")` | 1. Immediately understandable | Print statement |

**Score:** 60% immediately understandable, 20% understandable from context, 20% requires documentation. This is acceptable for a first program.

---

## 7. Operator Review

| Symbol | Uses | Conflict? | Verdict |
|--------|------|-----------|---------|
| `?` | Absence type prefix (`?User`) | Could conflict with propagation | KEEP if propagation uses `!`, CHANGE if propagation uses `?` |
| `!` | Logical NOT, failure propagation | **YES — critical conflict** | CHANGE propagation to `?` |
| `->` | Return type | No conflict | KEEP |
| `[]` | Generic type parameters, index access | Minor (different positions) | KEEP |
| `()` | Function parameters, tuples, unit | Minor (different positions) | KEEP |
| `{}` | Blocks, struct construction | Minor (different positions) | KEEP |
| `.` | Field access, method calls | No conflict | KEEP |
| `;` | Statement separator | No conflict | KEEP |
| `:` | Type annotations, named arguments | Minor (different positions) | KEEP |
| `::` | Type construction, module paths | Minor (overloaded) | KEEP with documentation |
| `&` | Shared reference (proposed) | No conflict | ADD |
| `&mut` | Exclusive reference (proposed) | No conflict | ADD |
| `=` | Assignment | No conflict | KEEP |
| `==`, `!=` | Equality | No conflict | KEEP |
| `<`, `>`, `<=`, `>=` | Ordering | No conflict | KEEP |
| `+`, `-`, `*`, `/`, `%` | Arithmetic | No conflict | KEEP |
| `&&`, `||` | Logical AND, OR | No conflict | KEEP |

---

## 8. Keyword Review

| Keyword | Purpose | Verdict | Reason |
|---------|---------|---------|--------|
| `fn` | Function declaration | KEEP | Short, unambiguous, familiar |
| `let` | Immutable binding | KEEP | Universal, clear |
| `mut` | Mutable modifier | KEEP | Clear, familiar from Rust |
| `pub` | Public visibility | KEEP | Terse, familiar |
| `type` | Type alias | RECONSIDER | Overloaded (means "type" generally) |
| `enum` | Sum type | KEEP | Standard |
| `struct` | Product type | KEEP | Standard |
| `impl` | Method implementation | KEEP | Standard |
| `import` | Module import | KEEP | Universal |
| `domain` | Execution domain | KEEP | AXIOM-specific, important |
| `call` | Cross-domain call | REMOVE | Replace with `::` domain qualification |
| `spawn` | Task creation | KEEP | Clear, AXIOM-specific |
| `await` | Task result | KEEP | Standard |
| `chan` | Channel type | RECONSIDER | Might be better as `Channel` |
| `match` | Pattern matching | KEEP | Standard |
| `if` / `else` | Conditional | KEEP | Universal |
| `while` | Loop | KEEP | Universal |
| `for` | Iterator loop | KEEP | Universal |
| `return` | Early return | KEEP | Universal |
| `break` | Exit loop | KEEP | Universal |
| `continue` | Next iteration | KEEP | Universal |
| `panic` | Unrecoverable error | KEEP | Clear |
| `unreachable` | Dead code | KEEP | Clear |
| `self` | Current instance | KEEP | Standard |
| `true` / `false` | Boolean literals | KEEP | Universal |

---

## 9. Syntax Consistency Review

| Pattern | Current | Consistent? | Issue |
|---------|---------|-------------|-------|
| Type construction | `User { ... }` | Yes | Consistent |
| Type alias | `type X = Y` | Yes | Consistent |
| Generic types | `List[T]` | Yes | Consistent |
| Generic functions | `fn first[T](...)` | Yes | Consistent |
| Module paths | `std.http` | Yes | Consistent |
| Type members | `User::new` | Yes | Consistent |
| Method calls | `user.name` | Yes | Consistent |
| Function calls | `add(1, 2)` | Yes | Consistent |
| Propagation | `expr!` | **Inconsistent** | `!` is also logical NOT |
| Absence | `?User` | **Inconsistent** | `?` could be propagation |
| Channel creation | `chan[Int]` | **Inconsistent** | Looks like type, not value |
| Domain calls | `call server fn(args)` | **Inconsistent** | Not a function call pattern |

---

## 10. Cross-Language Influence Review

| Construct | Resembles | Classification |
|-----------|-----------|---------------|
| `fn` | Rust | A. Familiarity benefit |
| `let` / `let mut` | Rust, ML | A. Familiarity benefit |
| `struct` | C, Rust, Go | C. Necessary convention |
| `enum` | C, Rust, Java | C. Necessary convention |
| `impl` | Rust | A. Familiarity benefit |
| `match` | Rust, ML | A. Familiarity benefit |
| `->` return type | Rust, Go, ML | C. Necessary convention |
| `?` absence prefix | Kotlin (`?`), Swift (`Optional`) | A. Familiarity benefit |
| `!` propagation | Rust (`?`) | B. Accidental imitation (should use `?`) |
| `domain` blocks | **AXIOM-specific** | D. AXIOM-specific improvement |
| `call server fn(args)` | **AXIOM-specific** | D. AXIOM-specific (but needs redesign) |
| `chan[T]` | Go (`chan T`), Rust (`channel`) | A. Familiarity benefit |
| `spawn` | Erlang, Go (`go`), Kotlin | A. Familiarity benefit |
| `.await` | JavaScript, Rust, C# | A. Familiarity benefit |
| `pub` | Rust | A. Familiarity benefit |
| `import` | Python, Java, JavaScript | C. Necessary convention |

---

## 11. Parser/Grammar Risks

| Risk | Severity | Description |
|------|----------|-------------|
| `!` ambiguity | **Critical** | Prefix `!` (NOT) vs postfix `!` (propagation) |
| `?T` vs `expr?` | Low | Type prefix vs expression postfix — distinguishable |
| `fn` overload | Low | Declaration vs closure — distinguishable by presence of name |
| `[]` overload | Low | Generic parameter vs index — distinguishable by context |
| `::` overload | Low | Type construction vs module path — distinguishable by left side |
| Semicolon rules | Medium | Last expression without semicolon is value — requires clear error messages |
| `match` parsing | Low | `=>` is distinct from `->` — no ambiguity |
| `domain` block | Low | `{ }` delimits the block — no ambiguity |
| Generic parsing | Medium | `List[T]` could be confused with index `list[T]` — need context |
| Named arguments | Low | `name: value` in call vs `name: Type` in declaration — distinguishable by position |

---

## 12. Formatter/LSP Risks

| Feature | Risk | Description |
|---------|------|-------------|
| Semicolon omission | Low | Formatter must track last expression in block |
| `?` prefix | Low | Syntax highlighter must distinguish type vs expression |
| `!` postfix | Low | Syntax highlighter must distinguish NOT vs propagation |
| `domain` blocks | Low | Standard block formatting |
| Generic types | Low | Standard bracket formatting |
| Cross-domain calls | Low | `::` is standard |
| Pattern matching | Low | Standard match formatting |

---

## 13. 10-Year Syntax Evolution Test

| Future Feature | Syntax Debt? | Notes |
|----------------|-------------|-------|
| Generics | No | `List[T]` is extensible |
| Traits/interfaces | No | `impl` blocks can be extended |
| Macros | Possible | `!` prefix could conflict with macro syntax |
| Compile-time execution | No | `comptime` keyword can be added |
| Reflection | No | `reflect` keyword can be added |
| Async/networking | No | `spawn`/`await` are extensible |
| Database abstractions | No | `db::query` pattern is extensible |
| FFI | No | `extern` keyword can be added |
| Embedded/system | Possible | Need `unsafe` or `raw` keyword for raw pointers |
| Distributed execution | Possible | `domain` blocks can be extended to remote domains |
| Additional domains | No | `domain custom_name { ... }` is extensible |

---

## 14. Recommended Changes

### Change 1: Replace `!` Propagation with `?`

**CURRENT:**
```axiom
let value = expr()!
```

**PROBLEM:**
`!` conflicts with logical NOT. Creates parsing ambiguity and visual confusion.

**RECOMMENDED:**
```axiom
let value = expr()?
```

**REASON:**
`?` is postfix-only, has no prefix meaning in AXIOM, and is familiar from Rust. The parser can distinguish `?T` (type prefix) from `expr?` (expression postfix) by context.

---

### Change 2: Rename Absence from `?T` to `maybe T`

**CURRENT:**
```axiom
fn find_user(id: i64) -> ?User { ... }
```

**PROBLEM:**
If propagation becomes `?`, then `?T` (absence) and `expr?` (propagation) use the same symbol in different positions. This is parseable but confusing for beginners.

**RECOMMENDED:**
```axiom
fn find_user(id: i64) -> maybe User { ... }
```

**REASON:**
`maybe` is a keyword, not punctuation. It is visually distinct from `?` propagation. The expression `find_user(1)?` is clean (propagate failure). To handle absence, use methods: `find_user(1)?.or(default)`.

**ALTERNATIVE:**
Keep `?T` for absence and use `!` for propagation. This avoids the keyword but keeps the `!` conflict. Not recommended.

---

### Change 3: Replace `call server fn(args)` with `server::fn(args)`

**CURRENT:**
```axiom
call server fetch_user(id)
```

**PROBLEM:**
`call` keyword is verbose, does not compose with generics or named arguments, and looks like a command rather than a function call.

**RECOMMENDED:**
```axiom
server::fetch_user(id)
```

**REASON:**
`::` is already used for type construction and module paths. Using it for domain qualification is consistent. `server::fetch_user(id)` reads naturally and composes with generics, method calls, and named arguments.

---

### Change 4: Add Ownership/Borrowing Syntax

**CURRENT:**
Proposal B does not include `&` or `&mut` syntax.

**PROBLEM:**
The language-core-spec requires borrowing, but Proposal B provides no syntax for it.

**RECOMMENDED:**
Add `&T` for shared references and `&mut T` for exclusive references:

```axiom
fn print_user(user: &User) { ... }
fn update_user(user: &mut User) { ... }
let r = &my_user
let rm = &mut my_user
```

**REASON:**
`&` is universally understood as "reference." It does not conflict with any existing AXIOM syntax. The borrow checker can enforce these at compile time.

---

### Change 5: Clarify Channel Creation

**CURRENT:**
```axiom
let ch = chan[Int]
```

**PROBLEM:**
`chan[Int]` looks like a type, not a value.

**RECOMMENDED:**
```axiom
let ch = chan::new[Int]()
```

**REASON:**
Consistent with `User::new { ... }` pattern. The `[T]` is a type parameter on the constructor. The `()` indicates construction.

---

## 15. Syntax Decisions Safe to Lock

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Function keyword | `fn` | Unambiguous, short, familiar |
| Binding keyword | `let` / `let mut` | Universal, clear |
| Return arrow | `->` | Universal, unambiguous |
| Block syntax | `{ }` | Universal |
| String literals | `"..."` | Universal |
| Char literals | `'...'` | Universal |
| Comment syntax | `//` and `/* */` | Universal |
| Integer literals | Decimal, `0x`, `0b`, `0o` | Standard |
| Float literals | `3.14`, `1.0e10` | Standard |
| Boolean literals | `true`, `false` | Universal |
| Field access | `.` | Universal |
| Index access | `[]` | Universal |
| Import keyword | `import` | Universal |
| Visibility | `pub` prefix | Terse, familiar |
| Generics | `List[T]` | No angle bracket ambiguity |
| Match keyword | `match` | Standard |
| Match arrow | `=>` | Standard |
| While keyword | `while` | Universal |
| For keyword | `for` | Universal |
| In keyword | `in` | Universal |
| Break/continue | `break`, `continue` | Universal |
| Return keyword | `return` | Universal |
| Struct keyword | `struct` | Standard |
| Enum keyword | `enum` | Standard |
| Impl keyword | `impl` | Standard |
| Type alias | `type X = Y` | Standard |
| Panic | `panic(msg)` | Clear |
| Unreachable | `unreachable` | Clear |
| True/false | `true`, `false` | Universal |

---

## 16. Syntax Decisions That Must Remain Open

| Decision | Why it must remain OPEN |
|----------|------------------------|
| Propagation operator (`?` vs `!` vs other) | `?` recommended but needs validation |
| Absence type syntax (`maybe T` vs `?T` vs other) | `maybe T` recommended but needs validation |
| Domain syntax (block vs annotation) | `domain { }` recommended but needs validation |
| Channel creation (`chan::new[T]()` vs `chan[T]` vs other) | Needs validation |
| Lambda syntax (`|x| expr` vs `fn(x) expr` vs other) | `|x|` recommended but needs validation |
| Generic syntax (`[T]` vs `<T>` vs other) | `[T]` recommended but needs validation |
| Borrowing syntax (`&T` vs other) | `&T` recommended but needs validation |
| Ownership transfer syntax (implicit vs explicit) | Implicit recommended but needs validation |
| Module path separator (`.` vs `::`) | `.` for imports, `::` for type members — needs validation |
| Named argument syntax (`name: value` vs `name=value`) | Needs validation |

---

## 17. Semantic Problems Revealed

1. **Failure + Absence combination**: `?User ! Error` is visually noisy. The language needs a clean way to express "might be absent OR might fail." This is a real semantic problem, not just syntax.

2. **Cross-domain borrowing**: Can a function in one domain borrow a value from another domain? The domain model says domains are separate compilation units. Borrowing across domains likely needs serialization, not references.

3. **Channel ownership**: Who owns a channel? If a channel is created in one domain and sent to another, what happens to ownership? The channel model needs explicit ownership rules.

4. **Task return type**: When a task returns `Int ! Error`, what does `handle.await` return? `Int ! Error`? Or does the `!` propagate automatically? The task + failure interaction needs clear semantics.

5. **Domain block scope**: Does `domain server { ... }` create a new scope? If not, how do names inside the block interact with names outside? The scope model needs clarification.

---

## 18. Final Recommendation

### PROPOSAL B STATUS: APPROVE WITH CHANGES

The core structure is sound. The following **5 changes** are required before grammar design:

| # | Change | Impact |
|---|--------|--------|
| 1 | Replace `!` propagation with `?` | Small — affects ~5% of code |
| 2 | Rename absence from `?T` to `maybe T` | Small — affects absence types |
| 3 | Replace `call server fn(args)` with `server::fn(args)` | Small — affects cross-domain calls |
| 4 | Add `&T` / `&mut T` borrowing syntax | Small — adds notation for existing semantics |
| 5 | Clarify channel creation as `chan::new[T]()` | Small — affects channel construction |

These changes are targeted, do not require redesigning the language, and resolve the critical and major problems identified in this stress test.

### SYNTAX LOCK CANDIDATES

After the 5 changes above are applied, the following become safe to lock:

- All literals (integers, floats, strings, chars, booleans)
- Block syntax `{ }`
- Comment syntax `//` and `/* */`
- Operator precedence
- `fn` for function declarations
- `let` / `let mut` for bindings
- `->` for return types
- `[]` for generics
- `.` for field/method access
- `::` for type construction and domain qualification
- `import` for module imports
- `pub` for visibility
- `match` / `=>` for pattern matching
- `if` / `else` for conditionals
- `while` / `for` / `in` for loops
- `break` / `continue` / `return` for control flow
- `struct` / `enum` / `impl` / `type` for type definitions
- `?` for failure propagation
- `maybe T` for absence types
- `&T` / `&mut T` for borrowing
- `spawn` / `.await` for concurrency
- `domain { }` for execution domains
- `panic` / `unreachable` for unrecoverable errors

### SYNTAX OPEN QUESTIONS

| # | Question | Options |
|---|----------|---------|
| 1 | Exact lambda syntax | `\|x\| expr` vs `fn(x) expr` vs other |
| 2 | Named argument syntax | `name: value` vs `name=value` |
| 3 | Match exhaustiveness syntax | Implicit (compiler checks) vs explicit (`_ =>`) |
| 4 | Struct update syntax | `User { ..existing, name: "new" }` vs other |
| 5 | Range syntax | `0..10` vs `0 to 10` vs `[0, 10)` |
| 6 | String interpolation | `format!()` vs `"$var"` vs other |
| 7 | Module re-export syntax | `pub import` vs `export` vs other |
| 8 | Generic constraints syntax | `fn first[T: Indexable]` vs other |
