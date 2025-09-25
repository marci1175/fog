import printf(input: string, ...): void;
import scanf(filter: string, buffer: array<uintsmall, 10>): int;

function return_0(): uint {
    return 2;
}

struct alma {
    nev: string,
    szin: int,
}

struct osztaly {
    diakok: array<string, 3>
}

function main(): int {
    # array<alma, 3> kosar = {alma { nev: "granny", szin: 1}, alma { nev: "finom", szin: 2}, alma { nev: "szhar", szin: 3}};

    # string szhar_alma = kosar[2].nev;

    # printf("Alma neve: %s", szhar_alma);
    
    array<array<osztaly, 2>, 4> osztalyok = {
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }}
    };

    printf("Hello");

    printf("Termeszporkolt: %s", osztalyok[1][1].diakok[0]);

    return 0;
}
