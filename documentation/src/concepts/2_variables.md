# Variables

## Creating Variables

Since the language is statically typed, every variable must have its type defined at compile time. Each variable must be declared either as a constant or a variable.

**Here is how one can define a variable with the aforementioned types:**

```fog
var int age = 23;
const string name = "marci1175";
const bool is_male = true;
```

Defining a struct may seem tricky at first, but it is no different from most languages. Every field must be manually initialized with its own default value.

```fog
struct person {
    age: int,
    name: string,
    is_male: bool,
}

person somebody = person { age: 23, name: "marci", is_male: true };
```

Accessing an enum variable is no different from other languages. The default type for an enum is a `uint` if not defined by the user.

```fog
struct Apple {
    color: float,
    name: string
}

enum<Apple> Apples {
    Idared = Apple { color: 1.0, name: "Idared" },
    Granny = Apple { color: 0.5, name: "Granny Smith" }
}

enum Numbers {
    One,
    Two,
    SixtySeven = 67
}

const string ida_name = Apples::Idared.name;
const int integer_zwei = Numbers::Two;

# Returns an error since the default type of an enum is uint and enums variants cannot be casted to a different type.
const int float_zwei = Numbers::Two as float;

# Valid
const int float_zwei = Numbers::Two as uint as float;
```
