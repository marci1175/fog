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

person somebody = person { age: 23, name: "marci", is_male: true, };
```
