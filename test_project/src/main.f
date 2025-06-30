import printf(msg: string, ...): void;
import getchar(): int;

function main(): int {
    loop {
        printf("Enter 'x' to get some candy!\n");

        int ch = getchar();
        
        if (ch == 120) {
            printf("Fatass\n");
        }
        else {
            printf("Why didnt you listen to me?\n");
        }
    }

    return 0;
}