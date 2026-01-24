external printf(str: string, ...): int;
external getchar(): int;

struct marci {
    a: int,    
}

trait nber {
    alszik(this, dur: int): void;
}

marci implements {
    pub function get_num(this, mul: int): marci {
        return marci { a: 924 };
    }
}

marci implements nber {
    pub function alszik(this, dur: int): void {
        print("Alszik %i", dur);
    }
}

pub function main(): int {
    marci q = marci { a: 22 };
    # fn arg check!!!
    printf("Get num: %i\n", q.get_num(10).a);

    return 0;
}