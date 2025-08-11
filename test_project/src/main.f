import printf(msg: string, ...): void;
import time(num: int): int;
import sleep(secs: uint): void;

function return_2(a: int): void {
    printf("Num is: %i\n", a);
}

function main(): int {
    if (3 > 8) {
        printf("Baj van tesomsz!");
    }

    return 0;
}