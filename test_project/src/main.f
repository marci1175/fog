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
    Alma szia = Alma { szam: 2, masik_szam: 43 };

    open_win("Window1");

    int en = 16;

    printf("Num %i", szia.masik_szam);

    return 0;
}
