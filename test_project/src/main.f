external printf(str: string, ...): int;

struct marci {
    a: int,    
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

pub function beszeltet |T <- majom| (lhs: T): int {
    int a = lhs.beszel();
    return a;
}

pub function main(): int {
    marci q = marci { a: 10 };

    beszeltet(q);

    return 0;
}