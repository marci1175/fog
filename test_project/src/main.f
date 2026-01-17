external printf(str: string, ...): int;
external getchar(): int;

struct marci {
    a: int,    
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