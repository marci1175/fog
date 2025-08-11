# Logic Gates and Comparisons

**Please note that this part of the languge is not finished, I cannot grant the reliability of the code that is provided. This part of the language is subject to change.**

Logic Gates work **almost** perfectly, the issue is with comparisons, as every comparison "trait" is hardcoded into the language. Im aiming to implement traits or something similar to rust's solution so that cmp traits are more flexible and easier to use.

**An example for using Logic Gates:**

```fog
import printf(msg: string): void;

function main(): int {
    if (3 > 8) {
        printf("Oh no! Math broke!");
    }
    else {
        printf("Oh yes! Math is didn't break!");
    }

    return 0;
}
```

The following comparison operators are implemented currently (for all types):

- `!=`
- `==`
- `>`
- `>=`
- `<`
- `<=`
