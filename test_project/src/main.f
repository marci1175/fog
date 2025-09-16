import printf(input: string, ...): void;

function return_0(): uint {
    return 2;
}

struct alma {
    nev: string,
    szin: int,
}

function main(): int {
    alma idared = alma { nev: "idared", szin: 69};

    array<uint, 5> marci = {90, 4, 5, 6, 7};

    # marci[2] = 100;

    printf("The number is %i %s", marci[2] as int, idared.nev);

    return 0;
}
