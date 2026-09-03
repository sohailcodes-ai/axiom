# AXIOM Grammar — MVP

This document defines the formal grammar for the AXIOM MVP. The grammar is written in a BNF-like notation.

---

## Lexical Rules

```
WHITESPACE  = ' ' | '\t' | '\n' | '\r'  (ignored between tokens)
COMMENT     = '//' [^\n]*               (single-line)
            | '/*' ... '*/'             (multi-line, not nested)

IDENT       = [a-zA-Z_][a-zA-Z0-9_]*
INT_LIT     = [0-9][0-9_]*              (decimal)
            | '0x' [0-9a-fA-F_]+        (hex)
            | '0b' [01_]+               (binary)
            | '0o' [0-7_]+              (octal)
FLOAT_LIT   = [0-9][0-9_]* '.' [0-9][0-9_]* ([eE][+-]?[0-9]+)?
BOOL_LIT    = 'true' | 'false'
CHAR_LIT    = '\'' [^'\\] '\''
            | '\'' ESCAPE '\''
STRING_LIT  = '"' [^"\\]* '"'
            | '"' ESCAPE_SEQUENCE* '"'
ESCAPE      = '\\' ['"\\ntr0]
```

---

## Keywords

```
Keywords (reserved):
    fn  let  mut  struct  enum  impl  type  pub
    if  else  while  for  in  match  return  break  continue
    import  domain  spawn  await  chan
    true  false  self  super
    panic  unreachable
    maybe  Some  None  Ok  Err
```

---

## Program Structure

```
program     = declaration*

declaration = struct_decl
            | enum_decl
            | type_alias
            | fn_decl
            | impl_block
            | domain_block
            | import_decl
            | const_decl
            | let_decl
```

---

## Declarations

### Struct

```
struct_decl = 'struct' IDENT '{' struct_field* '}'
struct_field = ['pub'] IDENT ':' type
```

### Enum

```
enum_decl   = 'enum' IDENT ['[' generic_params ']'] '{' enum_variant* '}'
enum_variant = IDENT ['(' type (',' type)* ')']
```

### Type Alias

```
type_alias  = 'type' IDENT '=' type
```

### Function

```
fn_decl     = ['pub'] 'fn' IDENT '(' param_list ')' ['->' type ['!' type]] block
param_list  = (param (',' param)*)?
param       = ['mut'] IDENT ':' type
```

### Impl Block

```
impl_block  = 'impl' IDENT '{' fn_decl* '}'
```

### Domain Block (MVP: simplified)

```
domain_block = 'domain' IDENT '{' declaration* '}'
```

### Import

```
import_decl = 'import' import_path
import_path = IDENT ('.' IDENT)*
```

### Constant

```
const_decl  = ['pub'] 'const' IDENT ':' type '=' expr
```

### Let (top-level)

```
let_decl    = ['pub'] 'let' ['mut'] IDENT [':' type] '=' expr
```

---

## Types

```
type        = primitive_type
            | reference_type
            | maybe_type
            | failure_type
            | function_type
            | tuple_type
            | array_type
            | named_type

primitive_type = 'i8' | 'i16' | 'i32' | 'i64'
               | 'u8' | 'u16' | 'u32' | 'u64'
               | 'f32' | 'f64'
               | 'bool' | 'char' | 'String'

reference_type = '&' type
               | '&' 'mut' type

maybe_type  = 'maybe' type

failure_type = type '!' type

function_type = 'fn' '(' type_list ')' '->' type
type_list   = (type (',' type)*)?

tuple_type  = '(' type (',' type)* ')'
            | '(' ')'

array_type  = '[' type ';' INT_LIT ']'

named_type  = IDENT ('[' type (',' type)* ']')?
```

---

## Expressions

```
expr        = literal
            | ident
            | field_access
            | index_access
            | call
            | unary_expr
            | binary_expr
            | if_expr
            | match_expr
            | block_expr
            | return_expr
            | struct_construct
            | closure
            | await_expr
            | propagation

literal     = INT_LIT | FLOAT_LIT | BOOL_LIT | CHAR_LIT | STRING_LIT | '()'

ident       = IDENT

field_access = expr '.' IDENT
index_access = expr '[' expr ']'
call        = expr '(' arg_list ')'
arg_list    = (expr (',' expr)*)?

unary_expr  = '-' expr
            | '!' expr
            | '&' expr
            | '&' 'mut' expr

binary_expr = expr binop expr
binop       = '==' | '!=' | '<' | '>' | '<=' | '>='
            | '+' | '-' | '*' | '/' | '%'
            | '&&' | '||'

if_expr     = 'if' expr block ['else' (if_expr | block)]

match_expr  = 'match' expr '{' match_arm* '}'
match_arm   = pattern '=>' expr ','?
pattern     = literal_pattern
            | ident_pattern
            | wildcard_pattern
            | tuple_pattern
            | struct_pattern
            | enum_pattern
            | or_pattern

literal_pattern = INT_LIT | FLOAT_LIT | BOOL_LIT | CHAR_LIT | STRING_LIT
ident_pattern   = IDENT
wildcard_pattern = '_'
tuple_pattern   = '(' pattern (',' pattern)* ')'
struct_pattern  = IDENT '{' (IDENT ':' pattern)* '}'
enum_pattern    = IDENT ['::'] IDENT ['(' pattern (',' pattern)* ')']
or_pattern      = pattern ('|' pattern)+

block_expr  = '{' statement* expr? '}'

return_expr = 'return' expr?

struct_construct = IDENT '{' (IDENT ':' expr)* '}'
                | IDENT '{' (IDENT ':' expr)* '..' expr '}'

closure     = '|' param_list '|' ['->' type] block
            | '|' param_list '|' expr

await_expr  = expr '.await'

propagation = expr '?'
```

---

## Statements

```
statement   = let_stmt
            | expr_stmt
            | return_stmt
            | break_stmt
            | continue_stmt

let_stmt    = 'let' ['mut'] IDENT [':' type] '=' expr
expr_stmt   = expr
return_stmt = 'return' expr?
break_stmt  = 'break' expr?
continue_stmt = 'continue'
```

---

## Blocks and Scopes

```
block       = '{' statement* expr? '}'
```

- A block is an expression
- The last expression (without semicolon) is the block's value
- If the block ends with a semicolon, the value is `()`
- Names introduced in a block are scoped to that block

---

## Operator Precedence (highest to lowest)

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `()` `[]` `.` `?.` `.await` | Left |
| 2 | `-` `!` `&` `&mut` | Right (unary) |
| 3 | `*` `/` `%` | Left |
| 4 | `+` `-` | Left |
| 5 | `<` `>` `<=` `>=` | Left |
| 6 | `==` `!=` | Left |
| 7 | `&&` | Left |
| 8 | `\|\|` | Left |
| 9 | `=` `+=` `-=` `*=` `/=` `%=` | Right |
| 10 | `?` (propagation) | Left |
| 11 | `!` (failure return) | Left (type position only) |

---

## Semicolon Rules

- Statements are separated by semicolons
- The last expression in a block may omit the semicolon to become the block's value
- A semicolon after the last expression discards the value (returns `()`)

---

## Parsing Notes

1. `fn name(...)` is a function declaration; `name(...)` is a function call
2. `IDENT { ... }` after a type name is struct construction; `IDENT { ... }` after `struct` keyword is struct definition
3. `[T]` after a type name is a generic parameter; `[expr]` after an expression is an index
4. `?` after an expression is propagation; `maybe T` in a type position is absence
5. `::` after a module/type name is member access
6. `&` before an expression is a borrow; `&` before a type is a reference type
7. `!` after `->` in a function signature is the failure type separator; `!` before an expression is logical NOT
