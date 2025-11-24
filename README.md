# Cupric Compiler

A compiler for the self-invented Cupric programming language, which is something of a hybrid between C and Rust.
Currently, the output of the compiler is LLVM IR, which can be further compiled into an executable with another program
such as `clang` or `llc`. Cupric source files use the `.cupr` file extension.

## Command Line Usage

```
Usage: compiler <package_path>

Arguments:
  <package_path>  Compile the package inside directory <package_path>

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Cupric Language

### Overview

The syntax and structure of Cupric is similar to that of Rust, whereas the semantics and capabilities of Cupric are
similar to those of C. Like Rust, most statements are also expressions, and mutability is explicit. Like C, there are
only pointers, no reference types, and there is no borrow checker or memory safety.

As is customary, here is a "hello world" program in Cupric:

```rust
foreign function main() -> i32 {
    libc::puts("Hello world!");
    0
}
```

### Features

#### Variables

Variables are defined with the `let` keyword. For example:

```rust
let immutable_variable: i32 = 5;
// OR
let mut mutable_variable: i32 = 5;
```

The value of a variable can only be modified after definition if the `mut` ("mutable") keyword is used.
If the type of the variable can be fully derived from the initial value, it can usually be omitted from the syntax.
If a variable is defined under the same name as an existing variable, the existing variable is *shadowed*. It still
exists, but its value can no longer be accessed in the scope the new variable was defined in.

#### Pointer Types

Pointer types are denoted by `*T` or `*mut T`. `*mut T` allows the underlying `T` to be modified, whereas `*T` does not.

#### Array Types

Array types can be either sized or unsized. Sized arrays are denoted by `[T; N]` where `N` is the desired length,
and unsized arrays are denoted by `[T]`. Unsized arrays have unknown length and must be accessed via pointer; for
instance, ASCII strings are often represented as `*[u8]` (immutable) or `*mut [u8]` (mutable).

#### Tuple Types

Tuple types hold members of varying types, much like structure types. Unlike structure types, members are referenced
by their order in the tuple rather than by name. For example, a tuple containing an integer and a string might have
type `(i32, *[u8])`.

Tuples are accessed using dot notation with the member index following the dot. For example, to extract the first
member (integer) from a variable `tuple` with the type above, use the expression `tuple.0`.

#### Structure Types

Structure types hold members of varying types, much like tuple types. Unlike tuple types, members are referenced
by their name in the structure definition rather than by order. These types are defined separately, and are given
a name which is used to refer to it.

```rust
struct MyStruct {
    integer: i32,
    string: *[u8],
}
```

When constructing a value for a structure type, order of the members does not matter since the names are used.

```rust
let my_value = MyStruct {
    string: "abc",
    integer: 123,
};
```

Structures are accessed using dot notation with the member name following the dot. For example, to extract the `integer`
member from `my_value`, use the expression `my_value.integer`.

#### Operations

Most common operations are implemented (arithmetic, comparison, bitwise, logical, assignment, etc.). For binary
arithmetic and comparison operations, both sides must have the same type.

Function calls can be used on function types.

The subscript (index) operation can be used on an array or pointer to array, sized or unsized. Unlike C, it cannot be
used on any pointer; unsized arrays are intended to fill this role.

For working with pointers, the dereference (`*`) and reference/address-of (`&`) operations behave like they do in C.

#### Compound Statements

Compound statements defined with curly braces can optionally end with an expression which becomes the resulting value
of the compound statement. As such, compound statements can be used as expressions themselves.

```rust
let result = {
    let mut thing = Thing::new();
    thing.modify();
    thing  // Tail expression
};
```

#### Conditionals (`if`...`else`)

Conditional `if`...`else` expressions work like one would expect.

```rust
// Works as a statement...
if (condition) {
    condition_was_true();
} else {
    condition_was_false();
}
// ...or as an expression!
let result = if (condition) {
    result_if_true()
} else {
    result_if_false()
};
```

#### Loops (`while`)

The `while` loop is currently the only supported form of loop.

```rust
while (condition) {
    do_something();
}
```

The `break` statement can be used to break out of the nearest loop, and the `continue` statement can be used to
continue to the next iteration of the nearest loop. In the future, these will accept an optional label.

#### Return

The `return` statement returns a value from the current function (or no value, if the function is `void`).

#### Foreign

Functions and global variables declared with the `foreign` keyword prevent the compiler from including the full path
for their name, thus allowing linkage with other libraries. This is used for the `main` function so it is recognized
by `libc`. If the `foreign` keyword is not used for `main`, even if it is located in the root module, its internal
symbol name will be `::main` (a prefix of `::` indicates the root module).

Structure types can also be declared `foreign` to indicate that they are *opaque*; that is, the composition of the
structure is not known. For example, `FILE` in C might be represented as follows:

```rust
foreign struct CFile;
```

Foreign structures can only be used via pointer since size and alignment information is not known.

#### Methods

Like Rust, type implementations are separate from the data. Methods for a type can be implemented using
the `implement` keyword. Within an `implement` block, any defined functions will be available as methods.
The `Self` type can be used to refer to the type currently being implemented.

```rust
struct Thing {
    value: u32,
}

implement Thing {
    function new() -> Self {
        Self { value: 0 }
    }

    function modify(self: *mut Self) {
        self.value += 1;
    }
}
```

Methods whose first parameter type is `Self`, `*Self`, or `*mut Self` may be called using dot notation, like
`my_thing.modify()`. In this case, `my_thing` is referenced as `*mut Thing` and passed as the first argument to
`modify`.

Any method can be called using the static notation, like `Thing::new()`. This is also true for instance methods; for
example, `my_thing.modify()` is equivalent to `Thing::modify(&my_thing)`.

#### Modules and Imports

Modules can be declared with the `module` keyword, and form a global namespace.

Symbols from other modules can be imported with the `import` keyword.

```rust
module a {
    module b {
        function hello() {}
    }
    import b::*;
}
import a::b::hello;
module c {
    import super::hello as my_function;
}
```
