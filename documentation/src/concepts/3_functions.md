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
//other.f
function return_2(): int {
    return 2;
} 

// main.f
import "other.f";
// You can also import source files on different paths like
// import "foo/bar/faz/test.f";
// import test::some_fn;
import printf(msg: string, ...): void;
import other::return_2;

function main(): int {
    int num = return_2();

    printf("Returned number: %i", num);

    return 0;
}
```

Note that we can also use variable args when constructing symbols for other functions. VarArgs cannot be used in a fog function.
