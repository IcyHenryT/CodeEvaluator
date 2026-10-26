# 03. Change a Value

A normal `let` variable is immutable. Write `let mut` when the same variable needs to change. An assignment updates an existing binding. Writing another `let` would instead introduce a new variable.

Compound assignments combine arithmetic and assignment. For example, `score *= 2;` doubles the stored value, and `score -= 3;` subtracts three. Likewise, `value += 1;` adds one to a stored value.

```rust
pub fn double_then_reduce(start: i32) -> i32 {
    let mut score = start;
    score *= 2;
    score -= 3;
    score
}
```

`double_then_reduce(5)` returns `7`: the stored score changes from `5` to `10` to `7`.

## Your task

Implement `pub fn increment_twice(start: i32) -> i32`.

Start with the input value, increase that same stored value by one twice, and return the result. Negative inputs and zero are valid.

### Expected results

| Call | Result |
| --- | ---: |
| `increment_twice(-5)` | `-3` |
| `increment_twice(-2)` | `0` |
| `increment_twice(0)` | `2` |
| `increment_twice(10)` | `12` |

### Syntax expectations

- Initialize the mutable local variable with `let mut value = start;`.
- Update it with two separate `value += 1;` assignments. Practice changing the existing binding rather than shadowing it or only calculating `start + 2`.
- Finish with the return expression `value`, without a semicolon.
