# 01. Give Values a Name

## Variables

A function takes inputs and returns a result. In `pub fn name(input: i32) -> i32`, `pub` makes the function available to callers such as the tests, and `i32` is an integer type that can hold negative numbers, zero, and positive numbers.

Inside a function, `let name = expression;` stores a value in a local variable. Rust can usually infer its type. Bindings are **immutable by default** aka you cannot assign a different value to them

### Worked example: triple a score

```rust
pub fn triple_score(score: i32) -> i32 {
    let multiplier = 3;
    score * multiplier
}
```

`triple_score(4)` returns `12`, and `triple_score(-2)` returns `-6`. The `let` statement ends with a semicolon. The final expression, `score * multiplier`, has **no semicolon**, so its value becomes the function's return value. Neither `score` nor `multiplier` needs to change.

## Your task

Implement `pub fn add_fee(price: i32) -> i32`.

Return the input price plus a fixed fee of `2`

### Expected results

| Call | Result |
| --- | ---: |
| `add_fee(-5)` | `-3` |
| `add_fee(-2)` | `0` |
| `add_fee(0)` | `2` |
| `add_fee(8)` | `10` |

You *must* create a variable
