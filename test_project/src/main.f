struct test {
    field1: int,
    field2: int,
}

function main(): int {
    test var1 = test { field1: 0, field2: 1, };
    
    return var1.field2;
}