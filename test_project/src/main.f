external printf(inp: string, ...): void;

import dependency-test::szia;

struct Alma {
    id: int,
    nev: string
}

pub function main(): int {
    array<Alma, 2> almak = { Alma { id: 2, nev: "Alma1"}, Alma { id: 1, nev: "Alma2"} };

    szia();

    return 0;
}