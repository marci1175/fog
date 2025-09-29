import printf(input: string, ...): int;

function main(): int {
    printf("Hello world!\n");
    
    array<array<int, 2>, 3> szamok = {{1, 2}, {1, 2}, {1, 2}};

    szamok[2][0] = 200;

    printf("Number: %i", szamok[2][0]);

    return 0;
}