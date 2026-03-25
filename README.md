# 🌙 Luno Programming Language

**Luno** is a general-purpose, interpreted scripting language inspired by Python. It features clean, indentation-based syntax with first-class functions, classes with inheritance, string interpolation, and pattern matching.

## 🚀 Features

- **Clean syntax** — Indentation-based blocks, no semicolons, no braces
- **First-class functions** — Closures, lambdas (`lam x => x * 2`), default args
- **Classes & inheritance** — `class Dog(Animal):` with `self`-based methods
- **String interpolation** — `` `Hello, ${name}!` `` with backtick strings
- **Pattern matching** — `match`/`case` expressions
- **Error handling** — `try`/`catch`/`finally` with custom error types
- **Immutability** — `let` for mutable, `const` for immutable bindings
- **Built-in functions** — `print`, `input`, `len`, `range`, `type`, and more
- **Interactive REPL** — Explore Luno interactively with `luno repl`

## 📥 Installation

```bash
git clone https://github.com/IMDevFlare/luno
cd luno
cargo build --release
```

The binary will be at `./target/release/luno`.

## 🛠 Usage

### Run a file

```bash
# Using the run subcommand
cargo run -- run examples/hello.luno

# Or directly
cargo run -- examples/hello.luno

# With the built binary
./target/release/luno run examples/hello.luno
```

Supported file extensions: `.luno`, `.ln`

### Start the REPL

```bash
cargo run -- repl
# or just
cargo run
```

```
🌙 Luno v0.1.0 — Interactive REPL
Type 'exit' or Ctrl+C to quit.

luno> print("Hello from Luno!")
Hello from Luno!
luno> let x = 42
luno> print(x * 2)
84
```

## 📜 Example

```luno
let name = "world"
print(`Hello, ${name}!`)

fn factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print(factorial(10))

class Animal:
    fn init(self, name: str):
        self.name = name
    fn speak(self):
        print(`${self.name} makes a sound.`)

class Dog(Animal):
    fn speak(self):
        print(`${self.name} barks!`)

let d = Dog("Rex")
d.speak()
```

**Output:**
```
Hello, world!
3628800
Rex barks!
```

## 📂 Project Structure

```
luno/
├── Cargo.toml           # Rust project manifest
├── src/
│   ├── main.rs          # CLI entry point (run / repl)
│   ├── lexer.rs         # Tokenizer with indentation tracking
│   ├── parser.rs        # Recursive-descent parser → AST
│   ├── interpreter.rs   # Tree-walking evaluator
│   └── repl.rs          # Interactive REPL loop
├── examples/
│   └── hello.luno       # Example program
├── GRAMMAR.md           # Complete language grammar reference
├── CHANGELOG.md         # Version history
└── TODO.md              # Roadmap and planned features
```

## 📄 License

[MIT](LICENSE)
