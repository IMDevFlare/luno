# 🌙 Luno Changelog

All notable changes to the Luno programming language will be documented in this file.

## [0.1.0] — 2026-03-24

### 🎉 Initial Release

#### Language Features
- Indentation-based block structure (no braces, no semicolons)
- Variable declarations with `let` (mutable) and `const` (immutable)
- Functions with `fn`, default parameters, variadic args (`*args`)
- Lambda expressions: `lam x => x * 2`
- Classes with `class`, single inheritance (`class Dog(Animal):`)
- Constructors via `fn init(self, ...):`
- `self` for instance reference
- `if` / `elif` / `else` conditionals
- `for x in iterable:` and `while condition:` loops
- `match` / `case` pattern matching
- `try` / `catch` / `finally` error handling
- Custom errors: `error MyError: "message"`
- `raise` to throw errors
- `import` and `from ... import ...` module support
- String interpolation with `` `Hello, ${name}!` ``
- Single-line (`#`) and multi-line (`## ... ##`) comments
- Optional type hints: `fn greet(name: str) -> str:`
- Closures and first-class functions

#### Data Types
- `int`, `float`, `str`, `bool`, `null`
- `list` (ordered, mutable)
- `map` (key-value pairs)
- `set` literals

#### Built-in Functions
- `print()`, `input()`, `len()`, `range()`, `type()`
- `str()`, `int()`, `float()`
- `abs()`, `min()`, `max()`

#### Standard Library
- `math` module: `sqrt`, `floor`, `ceil`, `sin`, `cos`, `pi`

#### Tooling
- Tree-walking interpreter in Rust
- Interactive REPL (`luno repl`)
- File runner (`luno run file.luno`)
- Direct file execution (`luno file.luno`)
- Support for `.luno` and `.ln` file extensions
