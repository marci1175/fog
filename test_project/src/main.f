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
    array<alma, 3> kosar = {alma { nev: "granny", szin: 1}, alma { nev: "finom", szin: 2}, alma { nev: "szhar", szin: 3}};

    alma szhar_alma = kosar[2];

    printf("Alma neve: %s", szhar_alma.nev);
    
    return 0;
}
