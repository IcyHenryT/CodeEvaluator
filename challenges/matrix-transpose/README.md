# Matrix Transpose

Implement `transpose(matrix: &[Vec<i64>]) -> Result<Vec<Vec<i64>>, RaggedMatrix>`.

Transpose a rectangular matrix so each input row becomes an output column. Empty and zero-column matrices return an empty matrix. Return `Err(RaggedMatrix)` when the rows have different lengths.

## Requirements

- Use at least 2 and fewer than 4 loops.
- Do not clone or modify the input.
