# 🌙 Luno Programming Language

Luno is a fast, readable programming language written in Rust. It replaces cryptic symbols with plain English keywords, making code feel like a natural conversation.

## 🚀 Key Features
- **Natural Language First**: Use `plus`, `minus`, `is greater than` or standard symbols.
- **Pythonic Structure**: Clean, no semicolons, indentation-focused (optional).
- **Rust-powered**: Safe, fast, and compiled for performance.

## 📥 Installation
```bash
git clone https://github.com/IMDevFlare/luno
cd luno
cargo build --release
```

## 🛠 Usage
Run a Luno file:
```bash
./target/release/luno examples/hello_world.lo
```

## 📜 Example
```luno
let name be "Alice"
say "Hello " ++ name

define add (a, b) {
    give back a plus b
}

say add(5, 10)
```