import printf(a: string, ...): int;

pub function factorial(x: uintlong): uintlong {
    uintlong a = 1;

    if (x != 0) {
        a = x * factorial(x - 1);
    }

    return a;
}

pub function main(): int {
    int marci = 0;

    marci = marci + 1;

    printf("Woah, marci: %i", marci);
    
    return 0;
}