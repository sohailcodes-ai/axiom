# AXIOM Syntax — Final (MVP)

This document defines the finalized syntax for the AXIOM MVP. All decisions here are applied from the Phase 2.5 stress test recommendations.

---

## 1. Bindings

```
let x = 42                    # immutable binding, type inferred
let x: i64 = 42              # immutable binding, type annotated
let mut x = 42               # mutable binding
x = 10                       # reassignment (only on mut bindings)
```

- `let` introduces an immutable binding
- `let mut` introduces a mutable binding
- Type annotation is optional when inference is possible
- Shadowing is allowed (a new `let` with the same name creates a new binding)

---

## 2. Types

### Primitives

```
i8, i16, i32, i64           # signed integers
u8, u16, u32, u64           # unsigned integers
f32, f64                     # floats
bool                         # true | false
char                         # Unicode scalar value
String                       # immutable UTF-8 string
```

### Compound

```
(Int, String)                # tuple type
(1, "hello")                 # tuple value
[Int; 5]                     # fixed-size array type
[1, 2, 3]                   # array value
List[Int]                    # dynamic list type
Map[String, Int]             # map type
```

### Structs

```
struct User {
    name: String
    age: i64
}

let user = User { name: "Alice", age: 30 }
let name = user.name         # field access
```

### Enums

```
enum Color {
    Red
    Green
    Blue
}

enum Result[T, E] {
    Ok(T)
    Err(E)
}

enum ApiError {
    NotFound
    BadRequest(String)
    DatabaseError(String)
}
```

### Type Aliases

```
type UserId = i64
type Callback = fn(i64) -> String
```

### References

```
&T                           # shared reference
&mut T                       # exclusive reference
&user                        # shared borrow of user
&mut user                    # exclusive borrow of user
```

---

## 3. Functions

```
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn greet(name: String) {
    print("Hello, " + name)
}

fn main() {
    let result = add(1, 2)
    greet("World")
}
```

- Functions have declared parameter types and return types
- Return type can be omitted when function returns `()` (unit)
- The last expression without a semicolon is the return value
- `return expr;` for early return

### Closures (MVP: minimal support)

```
let f = |x| x + 1
let result = f(5)
list.map(|x| x * 2)
```

---

## 4. Expressions

### Literals

```
42                          # integer
42i64                       # typed integer
3.14                        # float
3.14f32                     # typed float
0xff                        # hex
0b1010                      # binary
0o77                        # octal
1_000_000                   # separator
true                        # boolean
false                       # boolean
'a'                         # char
"hello"                     # string
()                          # unit
```

### Operators

```
# Arithmetic
+ - * / %

# Comparison
== != < > <= >=

# Logical
&& || !

# Assignment
= += -= *= /= %=
```

### Control Flow

```
# if expression
if x > 0 { x } else { -x }

# while loop
while i < 10 { i += 1 }

# for loop
for item in list { print(item) }

# break / continue
while true { break }
while true { continue }

# return
return 42
```

---

## 5. Pattern Matching

```
match color {
    Red => "stop"
    Green => "go"
    Blue => "sky"
}

match result {
    Ok(value) => print(value),
    Err(e) => print(e),
}

match value {
    0 => "zero",
    1 => "one",
    _ => "other",
}
```

- `match` must be exhaustive (compiler verifies)
- `_` is the wildcard pattern
- Patterns can destructure tuples, structs, and enums
- Each arm uses `=>` to separate pattern from expression

---

## 6. Absence (maybe T)

```
fn find_user(id: i64) -> maybe User {
    if id == 1 {
        Some(User { name: "Alice", age: 30 })
    } else {
        None
    }
}

let user = find_user(1)
match user {
    Some(u) => print(u.name),
    None => print("not found"),
}

let name = find_user(1).or(User { name: "Unknown", age: 0 }).name
let value = find_user(1)?.name   # propagate if absent (using ? on maybe)
```

- `maybe T` is the absence type
- `Some(value)` is the present variant
- `None` is the absent variant
- `.or(default)` provides a default value
- `.map(f)` transforms the present value
- `?` can be used on `maybe T` to propagate absence (returns `None` from the function)

---

## 7. Failure Return Types

```
fn parse_int(s: String) -> i64 ! ParseError {
    match s.to_i64() {
        Ok(n) => Ok(n),
        Err(_) => Err(ParseError.InvalidFormat(s)),
    }
}

fn process() -> i64 ! AppError {
    let a = parse_int("42")?    # propagate on failure
    let b = parse_int("10")?
    Ok(a + b)
}
```

- `-> T ! E` means the function returns `T` on success or `E` on failure
- `?` after an expression propagates the failure to the caller
- The failure type must match or be convertible to the caller's failure type
- `Ok(value)` constructs a success value
- `Err(error)` constructs a failure value

---

## 8. Ownership and Borrowing

```
# Ownership transfer (implicit)
fn consume(data: String) { ... }
let s = "hello".to_string()
consume(s)                  # s is moved, cannot use after this

# Shared borrow
fn print_user(user: &User) { ... }
print_user(&my_user)        # borrow, my_user still valid

# Exclusive borrow
fn update_user(user: &mut User) { ... }
update_user(&mut my_user)   # exclusive borrow

# Borrow in return
fn first(items: &List[i64]) -> &i64 {
    &items[0]
}
```

- Values are moved when passed to functions or assigned
- `&T` creates a shared reference (read-only)
- `&mut T` creates an exclusive reference (read-write)
- References must not outlive the value they reference
- In the MVP, the GC ensures safety; full borrow checking is deferred

---

## 9. Modules

```
import std.http
import std.json
import users

let user = users::find(1)
let response = http::get(url)
```

- `import` brings a module into scope
- `::` accesses members of a module or type
- Module names correspond to file paths

### Visibility

```
pub struct User { pub name: String, age: i64 }
pub fn public_function() { }
fn private_function() { }
```

- `pub` makes a declaration public
- Default visibility is private (module-scoped)

---

## 10. Execution Domains (MVP: single domain only)

```
domain server {
    fn handle_request(req: Request) -> Response ! HttpError {
        ...
    }
}

domain client {
    fn render(users: List[User]) -> Element {
        ...
    }
}
```

- `domain name { ... }` declares code that runs in a specific environment
- Cross-domain calls use `::` qualification: `server::handle_request(req)`
- The MVP supports a single domain (or no domain declaration)
- Multi-domain support is deferred to Phase 4

---

## 11. Concurrency (MVP: minimal support)

```
let handle = spawn {
    heavy_computation()
}
let result = handle.await

let ch = chan::new[i64]()
spawn {
    ch.send(42)
}
let value = ch.receive()
```

- `spawn { ... }` creates a concurrent task
- `handle.await` receives the task's result
- `chan::new[T]()` creates a typed channel
- `ch.send(value)` sends a value
- `ch.receive()` receives a value
- The MVP includes basic task/channel syntax; full structured concurrency is deferred

---

## 12. Comments

```
// single-line comment

/*
   multi-line comment
*/
```

---

## 13. Panic and Unreachable

```
panic("something went wrong")
unreachable
```

- `panic(msg)` terminates the current task with a message
- `unreachable` marks code that should never execute (panics if reached)
