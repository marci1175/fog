# Functions

## Creating Functions

To create functions, the `function` keyword can be used. This may be familiar to some from other languages that have similar keywords.

### Visibility

Every function requires a predetermined visibility attribute.

| Keyword | Explanation                                                                               |
| ------- | ----------------------------------------------------------------------------------------- |
| pub     | A public function is publicly available in the module.                                    |
| publib  | A public library function is available outside of the project when imported as a library. |
| private | A private function is only available in the module it is defined in.                      |

> Every source file serves as a different module.

**Function statement:**

```fog
<visibility> function <name>(<arguments>): <return type> {
    <body>
}
```

**Function definition example:**

```fog
pub function name(arg1: int, arg2: float) {
    // Function body
}

pub function name_2(arg1: int, arg2: float): int {
    // Function body

    return 0;
}
```

## Importing Functions

We can import functions from other source files or from libc. To import functions, we can use the `import` keyword.

**Here is how to import both types of functions:**

```fog
//other.f
pub function return_2(): int {
    return 2;
} 

// main.f
import "other.f";
// You can also import source files from different paths like
// import "foo/bar/faz/test.f";
// import test::some_fn;
import printf(msg: string, ...): void;
import other::return_2;

pub function main(): int {
    int num = return_2();

    printf("Returned number: %i", num);

    return 0;
}
```

> Note that we can also use variable arguments when constructing symbols for other functions. VarArgs cannot be used in a Fog function.
