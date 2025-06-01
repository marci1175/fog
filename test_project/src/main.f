import puts(msg: string): int;

struct test {
    inner: int,
}

function main(): int {
    puts("Hello World!");

    test a1 = test { inner: 32, };

    a1.inner = 23;

    return a1.inner;
}