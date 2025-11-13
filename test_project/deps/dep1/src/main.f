external printf(input: string, ...): int;

@feature "alma"
publib function kedvenc(): void {
    printf("Alma");
}

@feature "marci"
publib function kedvenc(): void {
    printf("Marci");
}

publib function printn(x: int): int {
    return printf("%i", x);
}