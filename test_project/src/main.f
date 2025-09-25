import printf(input: string, ...): void;
import scanf(filter: string, buffer: array<uintsmall, 10>): int;

function return_0(): uint {
    return 2;
}

struct osztaly {
    diakok: array<string, 3>
}

function main(): int {
    array<array<osztaly, 2>, 4> osztalyok = {
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }},
        {osztaly { diakok: {"marci30", "marci", "marci" } }, osztaly { diakok: {"marci30", "marci", "marci" } }}
    };

    int iq = 2000;

    printf("Hello\n");

    printf("Termeszporkolt: %s\n", osztalyok[return_0()][1].diakok[0]);

    if (iq == 2000) {
        osztaly haram = osztaly { diakok: {"Apad", "Anyad", "Cicad"} };

        printf("Hello: %s\n", haram.diakok[2]);
    }

    return 0;
}
