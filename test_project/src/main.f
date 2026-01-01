external printf(inp: string, ...): int;
# import dependency-test::szia;

pub function main(): int {  
    printf("csula\n");

    return 0;
}

pub function test(x: string): string {
    printf(x);
    printf(x);

    return x;
}

pub function test2(x: string): string {
    printf(x);
    printf(x);

    return x;
}