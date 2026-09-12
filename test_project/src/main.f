external function printf(str: string, ...): int;

import "helper.f";
import helper::helper_function1;
import helper::helper_function2;

public function main(): int {
    const int x = 1000;
    const int b = 31;

    if (x > 42) {
        printf("Hi: %i", helper_function1(47));
    } elseif (b > 35 || x == 1000) {
        printf("Hello: %i", helper_function2(100 / b));
    } else {
        printf("Did not work :()");
    }

    return 0;
}