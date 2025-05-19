import "sample.f";
import sample.apad;

import getchar(): int;
import greet(): void;

function main(): int {
    greet();

    int b = apad(x = 420);

    int a = getchar();
    
    return b;
}