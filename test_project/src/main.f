external printf(str: string, ...): int;

struct marci {
    a: int,
    c: float
}

trait majom {
    beszel(this): int;
}

marci implements {
    pub function beszel(this): int {
        printf("Szia batyus helyzeto: %i", this.a);

        return 0;
    }
}

pub function main(): int {
    marci q = marci { a: 200, c: 432 };

    q.beszel();

    q.a;
    q.c;

    return 0;
}

# investigate internal function not using the `this` argument in llvm ir
# Check float parsing