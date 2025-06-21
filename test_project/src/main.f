import printf(str: string, res: int): int;
import getchar(): int;
import gets(buf: string): int;

function main(): int {
    string buf = "0000000000000000000";

    int a = 0;

    loop {
        a = 23;
        
        printf("User input: %i\n", a);

        a = 3 - 1;

        printf("User input: %i\n", a);
    }

    return 0;
}