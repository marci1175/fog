external printf(inp: string, ...): int;
# import dependency-test::szia;

struct szia {
    nem: bool,
}

pub function main(): int {  
    printf("csula\n");

    return 0;
}

pub function test(x: string): string {
    printf(x);
    printf(x);

    return x;
}

pub function test2(x: string): szia {
    int marci = 0;
    szia hello = szia { nem: true };
    printf(x);

    return hello;
}