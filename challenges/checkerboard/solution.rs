pub fn checker_board(x: u64, y: u64) -> Vec<Vec<bool>> {
    let mut result: Vec<Vec<bool>> = Vec::with_capacity(x as usize);
    let mut odd: Vec<bool> = Vec::with_capacity(y as usize);
    let mut even: Vec<bool> = Vec::with_capacity(y as usize);

    for i in 0..y {
        odd.push(i % 2 == 0);
        even.push(i % 2 != 0);
    }

    for i in 0..x {
        if i % 2 == 0 {
            result.push(odd.clone());
        } else {
            result.push(even.clone());
        }
    }

    result
}
