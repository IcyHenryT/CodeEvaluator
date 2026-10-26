use submission::checker_board;

#[test]
fn one_by_one() {
    assert_eq!(checker_board(1, 1), vec![vec![true]]);
}

#[test]
fn three_by_one() {
    assert_eq!(
        checker_board(3, 1),
        vec![vec![true], vec![false], vec![true]],
    );
}

#[test]
fn one_by_three() {
    assert_eq!(checker_board(1, 3), vec![vec![true, false, true]]);
}

#[test]
fn three_by_three() {
    assert_eq!(
        checker_board(3, 3),
        vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ],
    );
}

#[test]
fn rectangular_board() {
    assert_eq!(
        checker_board(2, 4),
        vec![
            vec![true, false, true, false],
            vec![false, true, false, true],
        ],
    );
}

#[test]
fn empty_dimensions() {
    assert_eq!(checker_board(0, 3), Vec::<Vec<bool>>::new());
    assert_eq!(checker_board(3, 0), vec![vec![], vec![], vec![]]);
}
