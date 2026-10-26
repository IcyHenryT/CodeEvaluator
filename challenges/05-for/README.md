# 05. Count with for

A `for` loop visits each value in a sequence. For counting from `1` through `n`, use the **inclusive** range `1..=n`: the `=` means that `n` is included. In contrast, `1..n` stops before `n`. The range `1..=0` is empty, so its loop body does not run.

The loop supplies the next value automatically. Its binding is immutable by default and receives a new value each iteration. If a total must change across iterations, declare that separate binding with `let mut`.

### Worked example: multiply the numbers in a range

```rust
pub fn product_through(n: u32) -> u32 {
    let mut product = 1;
    for factor in 1..=n {
        product *= factor;
    }
    product
}
```

`product_through(4)` returns `24`, from `1 * 2 * 3 * 4`. `product_through(0)` returns the initial product, `1`, because there are no iterations. These example inputs are small enough that the multiplication fits in `u32`, the unsigned integer type used for nonnegative values.

`factor` does not need `mut`, because the loop provides each successive factor. `product` does need `mut`, because `*=` changes its stored value. The final expression `product` has no semicolon and returns the result declared by `-> u32`.

## Your task

Implement `pub fn sum_with_for(limit: u32) -> u32`.

Add every integer from `1` through `limit`, including `limit`. Return `0` for a limit of `0`. This is the same result as the previous lesson, but now practice using a `for` loop to handle the counting.

### Expected results

| Call | Result |
| --- | ---: |
| `sum_with_for(0)` | `0` |
| `sum_with_for(1)` | `1` |
| `sum_with_for(2)` | `3` |
| `sum_with_for(5)` | `15` |
| `sum_with_for(10)` | `55` |
| `sum_with_for(1000)` | `500500` |

### Syntax expectations

- Use a `for` loop over the inclusive range `1..=limit` to perform the summation, not just a formula or a different kind of loop.
- Initialize a mutable total to `0` and add each loop value to it.
- Let the range advance the loop value automatically; no separate counter update is needed.
- Return the total as the final expression, without a semicolon.
