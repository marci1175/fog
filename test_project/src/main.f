external function printf(str: string, ...): int;

import "helper.f";
import helper::helper_function1;

public function main(): int {
    const int x = 1000;
    const int b = 31;

    helper_function1(helper_function1(b));

    if (true) {
        return 2;
    }
    else {
        return 1;
    }
}