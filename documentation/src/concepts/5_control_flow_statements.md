# Control Flow Statements

The language offers multiple ways of controlling code flow. This includes basic concepts such as loops and code flow statements.

**An example of `loop` usage (which is similar to Rust's `loop` keyword):**

> Please note that loops are unstable in the current build of Fog due to faulty optimizations.

```fog
loop {
    # Do something

    break;
}
```

**Example for `while` and `for` usage:**

---
> **This is currently in development and may not be available in the latest edition of the compiler!**
---

> A `for` loop can only be used to iterate through a range of numbers; it does not support iterator objects.

**`While` statement:**

```fog
while (<condition>) {
    <body>
}
```

**`For` statement:**

```fog
for <local_variable> in (<range>, <step>) {
    <body>
}
```

**Example usage of both:**

```fog
int counter = 0;

while (counter < 10) {
    # Do something
}

array<int, 3> numbers = {1, 634, 4};

for idx in (0..2, 1) {
    int number = numbers[idx];
}
```
