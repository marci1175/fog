external printf(str: string, ...): int;
external getchar(): int;

struct marci {
    a: int,    
}

trait nber {
    alszik(this, dur: int): void;
}

marci implements {
    pub function get_num(this, mul: int): int {
        return 20 * mul;
    }
}

marci implements nber {
    pub function alszik(this, dur: int): void {
        print("Alszik %i", dur);
    }
}

pub function main(): int {
    marci q = marci { a: 22 };

    printf("Get num: %i\n", q.get_num(11));
    
    return 0;
}
