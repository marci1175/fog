external printf(str: string, ...): int;

struct marci {
    a: int,
    c: floatlong
}

trait majom {
    beszel(this): int;
}

trait abc {
    ligma(this, a: int): void;
}

trait asdasd {
    asd(this, a: int): void;
}


marci implements majom {
    pub function beszel(this): int {
        printf("Marci szama: %f\n", this.c);

        return 0;
    }
}

marci implements abc {
    pub function ligma(this, a: int): void {
        printf("A: %i\n", a);
    }
}

pub function test |T <- majom + abc + asdasd| (a: T): void {
    a.beszel();
    a.ligma(9000);
}

pub function main(): int {
    marci q = marci { a: 200, c: 432.2 };

    test(q);
    q.ligma(43);

    return 0;
}