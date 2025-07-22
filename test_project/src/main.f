import printf(msg: string, ...): void;
import time(num: int): int;
import sleep(secs: uint): void;

function return_2(a: int): void {
    printf("Num is: %i\n", a);
}

function main(): int {
    loop {
        int res = 1 + 2;
        int res2 = 6 - 9;
        if (res > res2) {
            printf("math is still intact!\n");
        }
        else {
            printf("math broke!\n");
        }
    }

    return 0;
}