import printf(msg: string, ...): void;
import time(num: int): int;
import sleep(secs: uint): void;

function main(): int {
    int app_start_time = time(0);

    printf("Seconds since epoch: %i\n", app_start_time);

    int destination_secs = app_start_time + 10;

    printf("Destination time: %i\n", destination_secs);
    
    loop {
        int time = time(0);
        int secs_left = destination_secs - time;

        printf("Seconds since epoch: %i\n", time);
        printf("Seconds left till destination time: %i\n", secs_left);
        
        sleep(1000);

        if (secs_left == 0) {
            break;
        }
    }

    printf("We have exited the loop body!!!\n");

    return 0;
}