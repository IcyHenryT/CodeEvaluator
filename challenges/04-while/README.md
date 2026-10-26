# 04. Loops with while

A `while` loop checks a boolean condition before each iteration. If the condition is true, the body runs. If not, the body does not run.

```rust
pub fn halve_until_small(start: u32) -> u32 {
    let mut value = start;
    while value > 3 {
        value /= 2;
    }
    value
}
```

`u32` is an unsigned integer type. This means that it holds zero and positive integers, not negative integers. Integer division discards any fractional part. For `halve_until_small(20)`, `value` goes from `20` to `10` to `5` to `2`, then the condition becomes false and the function returns `2`. For an input of `3`, the loop is skipped and the function returns `3`.

**Update the state inside a `while` loop.** Here `value /= 2;` makes progress toward a false condition. Without that update, an input above `3` would cause an infinite loop. A `while` loop does not stop itself automatically.

## Your task

Implement `pub fn sum_with_while(limit: u32) -> u32`.

Add every integer from `1` through `limit`, including `limit`. Return `0` when `limit` is `0`, since there are no numbers to add.

### Expected results

| Call | Result |
| --- | ---: |
| `sum_with_while(0)` | `0` |
| `sum_with_while(1)` | `1` |
| `sum_with_while(2)` | `3` |
| `sum_with_while(5)` | `15` |
| `sum_with_while(10)` | `55` |
| `sum_with_while(1000)` | `500500` |

### Syntax expectations

- Use a `while` loop to perform the summation, not just a formula or a different kind of loop.
- Keep a mutable total starting at `0` and a mutable counter starting at `1`.
- Continue while the counter is at most `limit`. Add the counter to the total and advance the counter on every iteration so the loop terminates.
- Return the total as the final expression, without a semicolon.
