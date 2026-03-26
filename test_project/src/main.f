external printf(str: string, ...): int;

struct marci {
    a: int,
    c: floatlong
}

struct doggo {
    thought: string
}

trait human {
    talk(this): int;
}

trait dog {
    bark(this): void;
}

marci implements human {
    pub function talk(this): int {
        printf("My float: %f\n", this.c);

        return 0;
    }
}

doggo implements dog {
    pub function bark(this): void {
        printf("The dog says: %s", this.thought);
    }
}

pub function do_something |H <- human, D <- dog| (human: H, dog: D): void {
    human.talk();
    dog.bark();
}

pub function main(): int {
    marci me = marci { a: 200, c: 432.2 };
    doggo my_dog = doggo { thought: "bark" };

    do_something(me, my_dog);

    return 0;
}