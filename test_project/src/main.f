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

    osztaly c_oszt = osztaly { diakok: {"marci", "marci", "marci" } };
    
    string marci = c_oszt.diakok[0]; 

    return 0;
}
