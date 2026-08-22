# Fog 🌫️

---

**Fog is a lightweight, high-performance programming language designed to be simple, flexible, and expressive.  
It focuses on minimalism, predictable semantics, and fast native compilation — with optional [tooling](https://github.com/marci1175/fog/tree/master/distributed-compiler) for large-scale workloads.**

---

![Endpoint Badge](https://img.shields.io/endpoint?url=https%3A%2F%2Fghloc.vercel.app%2Fapi%2Fmarci1175%2Ffog%2Fbadge)

## Features

| Feature | Status |
|--------|--------|
| LLVM Backend | Supported ✅ |
| Fog IR + LLVM IR Emission | Supported ✅ |
| Custom Types | Supported ✅ |
| Module System | Supported ✅ |
| Dependency System | Supported ✅ |
| Function Generics & Traits | Supported ✅ |
| Debug Information | Partially Supported ⚠️ |
| FFI (C ABI) | Partially Supported ⚠️ |
| Dynamic Memory Allocation | Planned 🔵 |
| Async / Tasks | Planned 🔵 |
| Incremental Compilation | Planned 🔵 |
| Full Standard Library | Planned 🔵 |

---

## Language Highlights

Fog offers a clean syntax designed around expressive power:

```fog
external println(lhs: string, ...);

pub function main(): int {
    array<int, 4> numbers = {1, 2, 3, 4};
    
    ptr<int> item_ptr = ref numbers[0];
    ptr<array<int, 4>> array_ptr = ref numbers;

    string name = "Alma";
    int number = wello(10, 24);

    println("Name: %s, Number: %i", name, number);

    return (deref item_ptr) - 1;
}

@feature "testing"
pub function wello(lhs: int, rhs: int): int {
    return lhs * (rhs - lhs);
}
```