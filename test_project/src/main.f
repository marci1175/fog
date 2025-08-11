import printf(msg: string, ...): void;
import "other.f";
import other::return_2;

function main(): int {
    return_2();

    return 0;
}