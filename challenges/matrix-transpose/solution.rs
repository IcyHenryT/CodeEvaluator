pub fn transpose(matrix: &[Vec<i64>]) -> Result<Vec<Vec<i64>>, RaggedMatrix> {
    let column_count = matrix.first().map_or(0, Vec::len);
    if matrix.iter().any(|row| row.len() != column_count) {
        return Err(RaggedMatrix);
    }

    let mut transposed = vec![Vec::with_capacity(matrix.len()); column_count];
    for row in matrix {
        for (column, value) in row.iter().enumerate() {
            transposed[column].push(*value);
        }
    }

    Ok(transposed)
}
