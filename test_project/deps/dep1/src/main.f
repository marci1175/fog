external printf(input: string, ...): int;
external hi_from_cpp(): void;

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

publib function hi_from_ffi(): void {
    hi_from_cpp();
}