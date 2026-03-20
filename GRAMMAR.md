# 🌙 Luno Grammar Specification

## 🔡 Basic Syntax

### Comments
```luno
-- This is a comment
# This is also a comment
```

### Variables
```luno
let <id> be <expr>
let <id> = <expr>
let <id>: <type> be <expr>
let <id1>, <id2> be <val1>, <val2>
```

### Functions
```luno
define <id> (<args>) {
    <body>
    give back <expr>
}
```

### Control Flow
```luno
if <condition> then {
    ...
} otherwise {
    ...
}
```

### Loops
```luno
repeat from <start> to <end> as <id> { ... }
for each <item> in <list> { ... }
while <condition> { ... }
```

### Strings
- Concatenation: `++`
- Operations: `uppercase of`, `length of`, `contains`, `replace`

### Operator Precedence (Highest to Lowest)
1. `()`
2. `of` (property/method access)
3. `not`
4. `*`, `/`, `times`, `divided by`
5. `+`, `-`, `plus`, `minus`, `++`
6. `>`, `<`, `>=`, `<=`, `is greater than`, `is less than`
7. `==`, `!=`, `is equal to`, `is not equal to`
8. `and`
9. `or`

## 📦 Objects & Classes
```luno
build <ClassName> (<fields>) {
    define <method> (...) { ... }
}
let <obj> = new <ClassName>(...)
```

## 🧬 Data Types
- `int`, `float`, `string`, `bool`, `list`, `map`
