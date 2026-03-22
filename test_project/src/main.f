external printf(str: string, ...): int;

struct marci {
    a: int,
    c: floatlong
}

trait majom {
    beszel(this): int;
}

trait tanydon {
    a(this): int;
}

marci implements majom {
    pub function beszel(this): int {
        # This unwraps a none in parsing
        printf("Marci szama: %f", this.c);

        return 0;
    }
}

pub function test |T <- majom| (a: T): void {
    a.beszel();
}

pub function main(): int {
    marci q = marci { a: 200, c: 432.2 };

    test(q);

    return 0;
}