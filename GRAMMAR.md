# 🌙 Luno Grammar Reference

Complete list of all keywords, operators, and syntax constructs in the Luno language.

## Keywords

| Keyword    | Description                          |
|------------|--------------------------------------|
| `let`      | Mutable variable declaration         |
| `const`    | Immutable variable declaration       |
| `fn`       | Function definition                  |
| `return`   | Return from function                 |
| `if`       | Conditional branch                   |
| `elif`     | Else-if branch                       |
| `else`     | Else branch                          |
| `for`      | For-in loop                          |
| `in`       | Iteration target                     |
| `while`    | While loop                           |
| `break`    | Break out of loop                    |
| `continue` | Skip to next loop iteration          |
| `match`    | Pattern matching                     |
| `case`     | Match case branch                    |
| `class`    | Class definition                     |
| `self`     | Instance self-reference              |
| `import`   | Import a module                      |
| `from`     | Import specific names from a module  |
| `try`      | Begin try block                      |
| `catch`    | Catch an error                       |
| `finally`  | Finally block (always runs)          |
| `raise`    | Raise an error                       |
| `error`    | Define a custom error type           |
| `lam`      | Lambda expression                    |
| `and`      | Logical AND                          |
| `or`       | Logical OR                           |
| `not`      | Logical NOT                          |
| `as`       | Alias in catch clause                |
| `true`     | Boolean true literal                 |
| `false`    | Boolean false literal                |
| `null`     | Null value literal                   |

## Operators

| Operator | Description         |
|----------|---------------------|
| `+`      | Addition / concat   |
| `-`      | Subtraction / negate |
| `*`      | Multiplication      |
| `/`      | Division            |
| `%`      | Modulo              |
| `**`     | Exponentiation      |
| `==`     | Equality            |
| `!=`     | Inequality          |
| `<`      | Less than           |
| `>`      | Greater than        |
| `<=`     | Less or equal       |
| `>=`     | Greater or equal    |
| `=`      | Assignment          |
| `+=`     | Add-assign          |
| `-=`     | Subtract-assign     |
| `*=`     | Multiply-assign     |
| `/=`     | Divide-assign       |
| `->`     | Return type arrow   |
| `=>`     | Lambda arrow        |
| `.`      | Attribute access    |
| `..`     | Range operator      |

## Data Types

| Type    | Example                        |
|---------|--------------------------------|
| `int`   | `42`, `0`, `-7`                |
| `float` | `3.14`, `-0.5`                 |
| `str`   | `"hello"`, `'world'`           |
| `bool`  | `true`, `false`                |
| `null`  | `null`                         |
| `list`  | `[1, 2, 3]`                    |
| `map`   | `{"key": "value"}`             |
| `set`   | `{1, 2, 3}`                    |

## Syntax Constructs

### Variables
```luno
let name = "Luno"
const PI = 3.14159
let typed: int = 42
```

### Functions
```luno
fn greet(name: str) -> str:
    return `Hello, ${name}!`

fn add(a, b = 0):
    return a + b
```

### Lambdas
```luno
let double = lam x => x * 2
```

### Control Flow
```luno
if x > 0:
    print("positive")
elif x == 0:
    print("zero")
else:
    print("negative")

for i in range(10):
    print(i)

while condition:
    break
```

### Match
```luno
match value:
    case 1:
        print("one")
    case 2:
        print("two")
```

### Classes
```luno
class Animal:
    fn init(self, name: str):
        self.name = name
    fn speak(self):
        print(`${self.name} makes a sound.`)

class Dog(Animal):
    fn speak(self):
        print(`${self.name} barks!`)
```

### Error Handling
```luno
try:
    raise MyError("oops")
catch MyError as e:
    print(e)
finally:
    print("cleanup")
```

### Imports
```luno
import math
from math import sqrt
```

### Comments
```luno
# Single-line comment

## This is a
   multi-line comment ##
```

### String Interpolation
```luno
let name = "world"
print(`Hello, ${name}!`)
```

## Built-in Functions

| Function   | Description                         |
|------------|-------------------------------------|
| `print()`  | Print values to stdout              |
| `input()`  | Read a line from stdin              |
| `len()`    | Length of str, list, or map         |
| `range()`  | Generate a list of integers         |
| `type()`   | Get the type name of a value        |
| `str()`    | Convert to string                   |
| `int()`    | Convert to integer                  |
| `float()`  | Convert to float                    |
| `abs()`    | Absolute value                      |
| `min()`    | Minimum of arguments                |
| `max()`    | Maximum of arguments                |
| `sqrt()`   | Square root (via `import math`)     |
| `floor()`  | Floor (via `import math`)           |
| `ceil()`   | Ceiling (via `import math`)         |
| `sin()`    | Sine (via `import math`)            |
| `cos()`    | Cosine (via `import math`)          |
| `pi()`     | Value of π (via `import math`)      |
