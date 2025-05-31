struct ligma {
    inner: int,
}

struct test {
    field1: int,
    field2: int,
    asd: ligma,
}

function main(): int {
    test var1 = test { field1: 0, field2: 69, asd: ligma { inner: 420, }, };
    
    return var1.asd.inner;
}