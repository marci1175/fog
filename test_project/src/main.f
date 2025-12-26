# import dependency-test::szia;
# pub function main(): int {
#     szia();
    
#     return 0;
# }

# struct a {
#     field1: array<int, 5>,
#     field2: array<int, 5>,
#     field3: array<int, 5>,
#     field4: array<int, 5>,
# }

# pub function main(): int {  
#     a b = a { field1: {0, 1, 2, 3, 4}, field2: {0, 1, 2, 3, 4}, field3: {0, 1, 2, 3, 4}, field4: {0, 1, 2, 3, 4} };

#     a.field1[2] = 400;

#     printf("%i", a.field1[2]);

#     return 0;
# }

external printf(inp: string, ...): int;

struct Apple {
    color: float,
    name: string
}

enum<Apple> Apples {
    Idared = Apple { color: 1.0, name: "Idared" },
    Granny = Apple { color: 0.5, name: "Granny Smith" }
}

enum Numbers {
    Zero,
    Two,
    SixtySeven = 67
}

pub function main(): int {
    Numbers null = Numbers::Zero;
    Numbers fu = Numbers::SixtySeven;
    Apple idared = Apples::Idared;

    printf("%i, %i, %s", null, fu, idared.name);

    return 0;
}