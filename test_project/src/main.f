import printf(input: string, ...): int;

function main(): int {
    printf("Hello world!\n");
    
    array<int, 3> szamok = {0, 0, 0};

    szamok[2] = 200;

    printf("Number: %i", szamok[2]);

    return 0;
}