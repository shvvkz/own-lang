# Own-Lang Compiler

## Introduction
Own-Lang is a custom programming language developed to explore compiler construction, parsing, and assembly code generation. This project includes a full pipeline from parsing Own-Lang syntax into an Abstract Syntax Tree (AST) and generating NASM x86_64 assembly code.

This README will guide you through setting up the project, understanding its components, and running examples provided in the `/own_files` directory.

## Features
- **Own-Lang Custom Syntax**: A simple, statically-typed language with basic control flow structures.
- **Compiler**: Translates Own-Lang code to NASM assembly.
- **NASM Code Generation**: Produces x86_64 assembly code that can be assembled and executed.
- **Functions and Variables**: Supports function definitions, variable declarations, and arithmetic expressions.
- **Loops & Conditionals**: Implements `for`, `while`, and `if` statements.

---

## Installation & Setup
### Requirements
Before you start, ensure you have the following installed:
- **Rust** (for compiling the Own-Lang compiler)
- **NASM** (for assembling the generated assembly code)
- **GCC or LD** (for linking the object files to an executable)

### Clone the Repository
```bash
git clone -b feat/compiler https://github.com/shvvkz/own-lang.git
cd own-lang
```

### Build the Compiler
```bash
cargo build --release
```

---

## Usage
### 1. Writing an Own-Lang Program
Create a file with an `.own` extension (e.g., `program.own`). Below is an example:

```own-lang
function add(a: int, b: int): int {
    let result: int = a + b;
    return result;
}

let num1: int = 1;
let num2: int = 2;
print(add(num1, num2));
```

### 2. Compiling Own-Lang Code
Run the compiler on your Own-Lang source file:
```bash
target/release/own-lang own_files/example.own
```
This generates an assembly file (`output.asm`).

### 3. Assembling and Running the Program
Compile the assembly code into an object file:
```bash
nasm -f elf64 output.asm -o output.o
```

Link it to create an executable:
```bash
gcc output.o -o output
```

Run the executable:
```bash
./output
```

---

## Own-Lang Syntax
### Variable Declaration
```own-lang
let x: int = 5;
let name: string = "Hello";
```

### Function Definition
```own-lang
function multiply(a: int, b: int): int {
    return a * b;
}
```

### Conditionals
```own-lang
if (x > 5) {
    print("x is greater than 5");
} else {
    print("x is 5 or less");
}
```

### Loops
#### For Loop
```own-lang
for (let i: int = 0; i < 10; i = i + 1) {
    print(i);
}
```

#### While Loop
```own-lang
let count: int = 0;
while (count < 5) {
    print(count);
    count = count + 1;
}
```

---
