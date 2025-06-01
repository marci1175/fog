import puts(msg: string): int;

struct asd {
    asd: string,
}

struct test {
    inner: int,
    asd: asd,
}

function main(): int {
    puts("Hello World!");

    test a1 = test { inner: 32, asd: asd {asd: "Inner value of asd.", }, };

    a1.inner = 23;

    puts(a1.asd.asd);

    return a1.inner;
}