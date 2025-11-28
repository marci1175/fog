external printf(inp: string, ...): void;

struct Alma {
    id: int,
    nev: string
}

pub function main(): int {
    array<Alma, 2> almak = { Alma { id: 2, nev: "Alma1"}, Alma { id: 1, nev: "Alma2"} };

    # almak[0].id = 10;

    # printf("%i", almak[0].id);

    return 0;
}