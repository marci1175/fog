external printf(str: string, ...): int;
external getchar(): int;

struct marci {
    a: int,    
}

trait nber {
    alszik(dur: int): void;
}

marci implements {
    pub function get_num(): int {
        return 0;
    }
}

marci implements nber {
    pub function alszik(dur: int): void {
        print("Alszik %i", dur);
    }
}

pub function main(): int {
    int a = 324;
    int b = 93;
    int c = 24;
    marci q = marci { a: 22 };
 
    if (q.a > b) {
        printf("Rip matek %i", b - a);
    }
    else {
        printf("Szamitas eredmenye: %i", b - q.a);
    }
 
    return 0;
}