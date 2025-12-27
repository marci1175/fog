external printf(inp: string, ...): int;

struct a {
    field1: array<int, 5>,
    field2: array<int, 5>,
    field3: array<int, 5>,
    field4: array<int, 5>,
}

pub function main(): int {  
    a b = a { field1: {0, 1, 2, 3, 4}, field2: {0, 1, 2, 3, 4}, field3: {0, 1, 2, 3, 4}, field4: {0, 1, 2, 3, 4} };

    b.field1[2] = 400;

    printf("%i", b.field1[2]);

    return 0;
}