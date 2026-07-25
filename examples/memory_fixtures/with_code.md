# Code fixture

```rust
fn main() {
    let answer = 42;
    println!("answer = {answer}");
    for i in 0..10 {
        let _ = i.saturating_add(1);
    }
}
```

A paragraph between fences.

```python
def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

print(fib(10))
```

```typescript
export function greet(name: string): string {
  return `hello, ${name}`;
}
```
