import printf(msg: string): void;

function main(): int {
    if (3 > 8) {
        printf("Oh no! Math broke!");
    }
    else {
        printf("Oh yes! Math is didn't break!");
    }

    return 0;
}