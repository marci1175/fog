import printf(str: string): int;

struct kg {
    inner: float,
}

function main(): int {
    kg suly = kg { inner: 103.12 };

    if (suly.inner > 30.0) {
        printf("Hello");
    } else {
        printf("Not Hello");
    }

    return 0;
}