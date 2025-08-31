import printf(input: string, ...): void;

function return_0(): int {
    return 0;
}

function main(): int {
    array<int, 5> marci = {2, 4, 5, 6, 7};

    int a = marci[return_0() as uintlong];

    printf("The number is %i", a);

    return 0;
}
