import printf(input: string, ...): int;

function main(): int {
    printf("Hello world!\n");
    
    array<array<int, 2>, 4> szamok = {
        { 0, 1 },
        { 0, 1 },
        { 0, 1 },
        { 0, 1 }
    };

    szamok[0][1] = 2;

    printf("Number %i", szamok[0][1]);

    return 0;
}