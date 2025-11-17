import dep1::hi_from_ffi;
import dep1::make_alma;
import dep1::open_win;
import dep1::printn;

struct Alma {
    szam: int,
    masik_szam: int,
}

external printf(a: string, ...): int;

pub function main(): int {
    open_win("Window1");

    return 0;
}
