import printf(msg: string, ...): void;
import time(num: int): int;
import sleep(secs: uint): void;

function main(): int {
    int curr_time = time(0);

    printf("Seconds since epoch: %i", curr_time);

    # Add 10 secs
    int destination_secs = curr_time + 10;

    loop {
        int secs_left = destination_secs - time(0);
        printf("Seconds left till destination time: %i", secs_left);
        
        sleep(1);

        if (secs_left == 0) {
            break;
        }
    }

    printf("We have exited the loop body!!!\n");

    return 0;
}