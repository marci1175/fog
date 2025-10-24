# Logic Gates and Comparisons

**Please note that this part of the language is not finished. I cannot guarantee the reliability of the code provided. This part of the language is subject to change.**

Logic gates work **almost** perfectly. The issue is with comparisons, as every comparison "trait" is hardcoded into the language. I am aiming to implement traits or something similar to Rust's solution so that comparison traits are more flexible and easier to use.

**An example of using logic gates:**

```fog
import printf(msg: string): void;

pub function main(): int {
    if (3 > 8) {
        printf("Oh no! Math broke!");
    }
    else {
        printf("Oh yes! Math didn't break!");
    }

    return 0;
}
```

The following comparison operators are currently implemented (for all types):

* `!=`
* `==`
* `>`
* `>=`
* `<`
* `<=`
