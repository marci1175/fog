import printf(a: string, ...): int;

### 12345
# 12345
@inline
pub function factorial(x: uintlong): uintlong {
    uintlong a = 1;

    if (x != 0) {
        a = x * factorial(#-> szia #-> x - 1);
    }

    return a;
}

@nofree
@feature "marci"
pub function main(): int {
    uintlong marci = 10;

    marci = factorial(marci);

    printf("Woah, marci: %ull", marci);
    
    return 0;
}