import printf(msg: string, ...): void;
import time(num: int): int;
import sleep(secs: uint): void;

function return_2(a: int): void {
    printf("Num is: %i\n", a);
}

function main(): int {
    loop {
        int res = 1 + 2;
        if (res > 2) {
            printf("math broke!");
        }
        else {
            printf("math is still intact!");
        }
    }

    return 0;
}