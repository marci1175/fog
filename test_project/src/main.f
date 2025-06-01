import puts(msg: string): int;

struct asd {
    inner: float,
}

function test(): int {
    return 10;
}

function main(): int {
    asd a = asd { inner: 23.2, };

    bool eq = a.inner == 23.2;

    return eq;
}