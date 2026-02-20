external printf(str: string, ...): int;
external getchar(): int;

trait nber {
    alszik(this, dur: int): void;
}


struct marci {
    a: int,    
}

trait majom {
    beszel(this, szo: string): void;
}


marci implements {
    pub function get_num(this, mul: int): int {
        this.a = 900;

        return this.a * mul;
    }
}

marci implements nber {
    pub function alszik(this, dur: int): void {
        print("Alszik %i", dur);
    }
}

pub function main(): int {
    marci q = marci { a: 10 };

    printf("Get num: %i\n", q.get_num(11));
    
    return 0;
}

pub function add |T <- nber, F <- nber| (lhs: T, rhs: int): int {
    return 0;
}