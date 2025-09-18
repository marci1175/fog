# Primitives

## Scalar types

Almost all of the basic scalar types are implemented in this language.

### Basic types

| Bits | Signed | Unsigned | Float |
|------|--------|----------|-------|
| 8-bit | - | uintsmall | - |
| 16-bit | inthalf | uinthalf | floathalf |
| 32-bit | int | uint | float |
| 64-bit | intlong | uintlong | floatlong |

### Additional types

`bool`: Used for storing a boolean value.

`void`: Used for indicating a function with no returned value

`string`: A string variable can be used to store text. The language handles strings as a pointer to an array. __(This comes into play when interacting with FFI)__

`array<T, L>`: An array can be used to store multiple values in the same variable. An array has a pre-determined type and length, as indicated by the generic `T` and `L`. A length can only be an `int`.

### Custom Types

Structs can also be created by the user via the `struct` keyword. Structs cannot contain themselves.
Definining a struct is similar to how one would do it in other languages.

```fog
struct my_struct {
    field1: int,
    field2: string,
    field3: bool,
}
```

Enums are also available and are accessible via the `enum` keyword. Enums can also serve as a `Default` value for any type.

```fog
enum Weekdays {
    Monday,
    Tuesday,
    Friday,
}

enum FavWords<string> {
    choco = "Chocolate",
    banana = "Banana",
}
```
