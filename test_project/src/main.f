external printf(str: string, ...): int;
external getchar(): int;

struct marci {
    a: int,    
}

trait majom {
    beszel(this): int;
}

marci implements {
    pub function get_num(this, mul: int): int {
        this.a = 900;

        return this.a * mul;
    }

    pub function beszel(this): int {
        printf("Szia batyus helyzeto: %i", this.a);

        return 0;
    }
}

pub function add |T <- majom| (lhs: T): int {
    int a = lhs.beszel();
    return a;
}

pub function main(): int {
    marci q = marci { a: 10 };

    # This shit does NOT work, llvm is doing something shady here lol pls investigate codegen
    # Must be some sort of a memory issue
    # Fn isnt called when not stored and is with incorrect arguments inside
    # int a = q.beszel();

    add(q);

    return 0;
}
