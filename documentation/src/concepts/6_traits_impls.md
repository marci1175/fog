# Traits and (trait) implementations

The language supports the basic concepts of traits and OOP ([Object-oriented programming](https://en.wikipedia.org/wiki/Object-oriented_programming)).

> Additional items or features might be modified in later updates

## Traits

Traits are basically a set of functionality that a struct can implement.

Traits are defined as such:

```fog
<visibility> trait <name> {
    function <name>(<arguments>): <return type>;
}
```

Functions can take types that implement a certain trait (or set of traits in later updates) as an argument, like so:

```fog
trait walks {
    function walk(this, set_walk: bool);
    function get_walk(this);
}

pub function generic(m: walks): int {
    if m.get_walk() {
        return 1;
    }
    else {
        return 0;
    }
}
```

## Implementations

Implementations are for structs and custom items to implement custom functionality and to support [OOP](https://en.wikipedia.org/wiki/Object-oriented_programming).

Impl bodies are defined as such:

```fog
<struct name> implements <trait (optional)> {
    pub foo(this, rhs: int): int {
        return this.inner * rhs;
    }

    pub bar(lhs: int, rhs: int): int {
        return lhs + rhs;
    }
}
```

Implementations can be accessed similarly to other languages.

```fog
struct math {
    inner: int
}

math implements {
    pub foo(this, rhs: int): int {
        return this.inner * rhs;
    }

    pub bar(lhs: int, rhs: int): int {
        return lhs + rhs;
    }
}

trait calculate {
    function cmp(this, rhs: int): bool;
}

math implements calculate {
    function cmp(this, rhs: int): bool {
        return this.inner == rhs;
    }
}

pub function main(): int {
    math bar = math { inner: 100 };

    int calc1 = bar.foo(2);
    int calc2 = math::bar(202, 3);
    
    # This will return 405
    return calc1 + calc2;
}
```
