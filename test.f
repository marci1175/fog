import getchar(): int;
import putchar(char: int): int;

function main(): int {
    int a = 23;

    putchar(char = a);
    int get_char_res = getchar();

    return get_char_res;
}