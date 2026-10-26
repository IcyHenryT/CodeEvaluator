# 02. Make a Choice

## Learn: conditions

An `if` expression chooses between two branches using a boolean condition. Comparisons such as `<`, `>`, and `==` produce a boolean. With an `else` branch, `if` can produce the value that a function returns. Both branches must produce the same type.

### Worked example: replace a negative number with zero

```rust
pub fn nonnegative(value: i32) -> i32 {
    if value < 0 {
        0
    } else {
        value
    }
}
```

`nonnegative(-4)` returns `0`, while `nonnegative(6)` returns `6`. Here `i32` permits negative numbers.

Each branch ends in a value without a semicolon. The whole `if` is the function's final expression, also without a semicolon, so the selected value is returned.

## Your task

Implement `pub fn bigger(left: i32, right: i32) -> i32`.

Return the larger input using an `if` expression. If the inputs are equal, return that common value.

### Expected results

| Call | Result |
| --- | ---: |
| `bigger(3, 8)` | `8` |
| `bigger(8, 3)` | `8` |
| `bigger(-7, -2)` | `-2` |
| `bigger(-5, 0)` | `0` |
| `bigger(-4, -4)` | `-4` |
| `bigger(0, 0)` | `0` |

### Syntax expectations

- Use an `if` expression with an `else` branch to choose between `left` and `right`.
- Let the branches produce the chosen input and use the whole `if` as the return expression, without a trailing semicolon.
