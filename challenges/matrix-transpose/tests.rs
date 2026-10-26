use submission::{transpose, RaggedMatrix};

#[test]
fn transposes_square_matrix() {
    let matrix = vec![vec![1, 2], vec![3, 4]];

    assert_eq!(transpose(&matrix), Ok(vec![vec![1, 3], vec![2, 4]]));
}

#[test]
fn transposes_wide_matrix() {
    let matrix = vec![vec![1, 2, 3], vec![4, 5, 6]];

    assert_eq!(
        transpose(&matrix),
        Ok(vec![vec![1, 4], vec![2, 5], vec![3, 6]]),
    );
}

#[test]
fn transposes_tall_and_singleton_matrices() {
    assert_eq!(
        transpose(&[vec![1], vec![2], vec![3]]),
        Ok(vec![vec![1, 2, 3]]),
    );
    assert_eq!(transpose(&[vec![-7]]), Ok(vec![vec![-7]]));
}

#[test]
fn accepts_empty_matrix() {
    assert_eq!(transpose(&[]), Ok(Vec::<Vec<i64>>::new()));
}

#[test]
fn accepts_rectangular_matrix_with_zero_columns() {
    assert_eq!(transpose(&[vec![], vec![]]), Ok(Vec::<Vec<i64>>::new()));
}

#[test]
fn rejects_ragged_matrices() {
    assert_eq!(transpose(&[vec![1, 2], vec![3]]), Err(RaggedMatrix));
    assert_eq!(transpose(&[vec![], vec![1]]), Err(RaggedMatrix));
}

#[test]
fn does_not_modify_the_input() {
    let matrix = vec![vec![1, 2, 3]];
    let original = matrix.clone();

    let _ = transpose(&matrix);
    assert_eq!(matrix, original);
}
