external printf(str: string, ...): int;

struct marci {
    a: int,
    c: floatlong
}

trait majom {
    beszel(this): int;
}

marci implements {
    pub function beszel(this): int {
        # This unwraps a none in parsing
        printf("Marci szama: %f", this.c as floatlong);

        return 0;
    }
}

pub function main(): int {
    marci q = marci { a: 200, c: 432.2 };

    q.beszel();

    return 0;
}

# Check float parsing