# Variables

## Creating Variables

Since the language is statically typed, every variable has to have its type defined at compile time. Initializing a variable is not crucial, as variables get a default value if left unintialized by the user.

__Here is how one can define a variable with the aforementioned types.__

```fog
int age = 23;
string name = "marci1175";
bool is_male = true;
```

Defining struct may seem tricky at first, but they are no different from most languages.
Every field has to be manually initialized with their own default value.

```fog
struct person {
    age: int,
    name: string,
    is_male: bool,
}

person somebody = person { age: 23, name: "marci", is_male: true };
```

Accessing an enum variable is no different from other languages. The default type for an enum is an `int` if not defined by the user.

```fog
struct Apple {
    color: float,
    name: string
}

enum Apples<Apple> {
    Idared = Apple { color: 1.0, name: "Idared" },
    Granny = Apple { color: 0.5, name: "Granny Smith" }
}

enum Numbers {
    One,
    Two,
    SixtySeven = 67
}

string ida_name = Apples::Idared.name;
int integer_zwei = Numbers::Two;
int float_zwei = Numbers::Two as float;
```
