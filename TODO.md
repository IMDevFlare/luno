# 🌙 Luno TODO / Roadmap

Planned features and improvements for future versions.

## v0.2.0 — CLI & File Handling

- [ ] CLI `luno <file>.ln` shorthand (currently supported)
- [ ] `lunoc` compiler frontend (compile to bytecode)
- [ ] `luno fmt` — auto-formatter for `.luno` files
- [ ] `luno check` — static type checker (using type hints)
- [ ] Better error messages with line/column numbers
- [ ] Source maps for error tracing

## v0.3.0 — Standard Library

- [ ] `io` module — `read_file()`, `write_file()`, `file_exists()`
- [ ] `string` module — `split()`, `join()`, `replace()`, `trim()`, `starts_with()`, `ends_with()`
- [ ] `list` module — `sort()`, `reverse()`, `map()`, `filter()`, `reduce()`
- [ ] `json` module — `parse()`, `stringify()`
- [ ] `os` module — `env()`, `args()`, `exit()`
- [ ] `time` module — `now()`, `sleep()`

## v0.4.0 — Language Features

- [ ] Destructuring: `let [a, b] = [1, 2]`
- [ ] Spread operator: `let combined = [...list1, ...list2]`
- [ ] List comprehensions: `[x * 2 for x in range(10)]`
- [ ] Dict comprehensions: `{k: v for k, v in pairs}`
- [ ] Multi-return / tuple unpacking
- [ ] `with` statement for resource management
- [ ] Decorators: `@decorator`
- [ ] Enum types
- [ ] Interfaces / traits

## v0.5.0 — Performance & Tooling

- [ ] Bytecode compilation (`lunoc`)
- [ ] Stack-based VM for faster execution
- [ ] Package manager (`luno pkg install`)
- [ ] Language server (LSP) for editor support
- [ ] Syntax highlighting for VS Code, Sublime, Vim
- [ ] WASM compilation target

## Backlog

- [ ] Async/await support
- [ ] Coroutines / generators (`yield`)
- [ ] Regular expression literals
- [ ] Pattern matching with destructuring
- [ ] Operator overloading via clean method names
- [ ] Multi-line string literals (triple backtick)
- [ ] Numeric separators: `1_000_000`
- [ ] Binary/hex/octal literals: `0b1010`, `0xFF`, `0o77`
- [ ] Interactive debugger
- [ ] Test framework built-in (`luno test`)
