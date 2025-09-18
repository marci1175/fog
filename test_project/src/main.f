import printf(input: string, ...): void;
import scanf(filter: string, buffer: array<uintsmall, 10>): int;

function return_0(): uint {
    return 2;
}

struct alma {
    nev: string,
    szin: int,
}

function main(): int {
    array<uintsmall, 10> buf = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0};
    
    scanf("%s", buf);

    printf("Hello %i", buf[2]);
    
    return 0;
}
