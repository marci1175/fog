import printf(input: string, ...): int;

struct fing {
    asd: int
}

function finggen(): fing {
    return fing { asd: 35 };
}

function main(): int {
    fing marci = finggen();

    printf("Number %i", marci.asd);

    return 0;
}