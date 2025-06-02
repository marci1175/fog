import puts(msg: string): int;

struct asd {
    inner: float,
}

function test(): float {
    return 10.23;
}

function main(): int {
    asd a = asd { inner: 23.2, };

    bool eq = a.inner == 23.2;

    return 1;
}