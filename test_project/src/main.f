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

    alma idared = marci[2];

    printf("The name is %s", idared.nev);

    return 0;
}
