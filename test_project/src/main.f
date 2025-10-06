import printf(input: string, ...): int;

struct fing {
    asd: int
}

function finggen(): fing {
    fing tmp = fing { asd: 0 };
    return tmp;
}

# [debug_attr: "Main function"]
# [hot]
# [no_mangle]
function main(): int {
    int szamok = 2;

    fing marci = finggen();

    printf("Number %s", szamok);

    return 0;
}