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

    array<alma, 5> marci = {alma { nev: "idared", szin: 69}, alma { nev: "idared", szin: 69}, alma { nev: "idared", szin: 69}, alma { nev: "idared", szin: 69}, alma { nev: "idared", szin: 69}};

    # marci[2] = 100;

    printf("The name is %s", marci[2].nev);

    return 0;
}
