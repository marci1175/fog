import printf(str: string, inp: int): int;
import srand(seed: int): int;
import time(since: int): int;
import rand(): int;
import gets(): int;

function main(): int {
    srand(time(0));

    loop {
        printf("Random number: %i\n", rand());
    }

    return 0;
} 