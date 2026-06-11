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
| Custom PE/COFF Linker | Supported ✅ |
| Distributed Build Infrastructure | Supported ✅ |
| Rich Error Diagnostics | Supported ✅ |
| Fog IR + LLVM IR Emission | Supported ✅ |
| Custom Types | Supported ✅ |
| Module System | Partially Supported ⚠️ |
| Debug Information | Partially Supported ⚠️ |
| FFI (C ABI) | Partially Supported ⚠️ |
| Cross-Compilation | Partially Supported ⚠️ |
| Dependency System | Partially Supported ⚠️ |
| Function Generics & Traits | Planned 🔵 |
| Dynamic Memory Allocation | Planned 🔵 |
| Async / Tasks | Planned 🔵 |
| Incremental Compilation | Planned 🔵 |
| Full Standard Library | Planned 🔵 |

---

## Language Highlights

Fog offers a clean syntax designed around expressive power:

```fog
external println(lhs: string, ...);

pub function add(a: int, b: int): int {
    return a + b;
}

pub function main() {
    int x = add(10, 20);
    println("%i", x);
}
```

🚧Disclaimer🚧
The project is being recoded and has been put on hold until 2026. 07. 01.
