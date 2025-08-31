import printf(input: string, ...): void;

function return_0(): int {
    return 2;
}

function main(): int {
    array<int, 5> marci = {90, 4, 5, 6, 7};

    int a = marci[return_0() as uint];

    printf("The number is %i", a);

    return 0;
}
