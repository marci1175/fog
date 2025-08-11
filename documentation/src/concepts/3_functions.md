# Functions

## Creating functions

For creating functions, the `function` keyword can be used. This may be familiar to some from other languages which have similar keywords.

__Function definition example:__

```fog
function name(arg1: int, arg2: float) {
    // Function body
}

function name_2(arg1: int, arg2: float): int {
    // Function body

    return 0;
}
```

## Importing functions

We can import functions from other source files, or from libc. For importing function we can use the `import` keyword.

__Here is how to import both types of functions:__

```fog
import "path_to_src_file/other.f";
import printf(msg: string, ...): void;

function main(): int {
    

    return 0;
}
```

Note that we can also use variable args when constructing symbols for other functions. VarArgs cannot be used in a fog function.
