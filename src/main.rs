use std::io;

use anyhow::Context;
use csv::StringRecord;

fn main() -> anyhow::Result<()> {
    let (puzzle, table_size) = read_input().context("error retrieving data")?;

    let TableSize {
        cols: num_cols,
        rows: num_rows,
    } = table_size;

    let mut col_headers: Vec<Vec<usize>> = Vec::new();
    let mut row_headers: Vec<Vec<usize>> = Vec::new();
    compute_headers(
        &mut col_headers,
        &mut row_headers,
        num_cols,
        num_rows,
        &puzzle,
    )?;

    let mut grid = vec![vec!["2"; num_cols]; num_rows];

    iterate(&mut grid, num_cols, num_rows, &col_headers, &row_headers)?;

    let mut solvable = true;
    for r in 0..num_rows {
        for c in 0..num_cols {
            if grid[r][c] == "2" {
                print!("X ");
                solvable = false;
            } else {
                print!("{} ", grid[r][c]);
            }
        }
        println!();
    }

    if solvable {
        println!("\nSolvable!");
    } else {
        println!("\nNot Solvable!");
    }

    Ok(())
}

struct TableSize {
    cols: usize,
    rows: usize,
}

fn read_input() -> Result<(Vec<StringRecord>, TableSize), anyhow::Error> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(io::stdin());

    let first_row = rdr
        .records()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no records found."))??;

    let table_size = TableSize {
        cols: first_row[0].parse()?,
        rows: first_row[1].parse()?,
    };

    // Next collect all records from the input.
    let data = rdr.records().collect::<Result<Vec<StringRecord>, _>>()?;

    // Make sure the number of rows we collected is equal to the number expected.
    if data.len() != table_size.rows {
        anyhow::bail!("number of rows found does not match the expected input.");
    }

    // We can check every row for the column count
    // It may be necessary to only check the first?
    if data.iter().any(|row| row.len() != table_size.cols) {
        anyhow::bail!("the number of columns specified in the header did not match the number found in all rows.");
    }

    Ok((data, table_size))
}

fn compute_headers(
    col_headers: &mut Vec<Vec<usize>>,
    row_headers: &mut Vec<Vec<usize>>,
    num_cols: usize,
    num_rows: usize,
    puzzle: &Vec<StringRecord>,
) -> anyhow::Result<()> {
    for c in 0..num_cols {
        let mut header: Vec<usize> = Vec::new();
        let mut count: usize = 0;
        for r in 0..num_rows {
            if puzzle[r][c] == *"0" && count > 0 {
                header.push(count);
                count = 0;
            } else if puzzle[r][c] == *"1" {
                count += 1;
            } else if puzzle[r][c] != *"0" {
                anyhow::bail!("data corrupted: bad cell data");
            }
        }
        if count > 0 || header.is_empty() {
            header.push(count);
        }

        col_headers.push(header);
    }

    for r in 0..num_rows {
        let mut header: Vec<usize> = Vec::new();
        let mut count: usize = 0;
        for c in 0..num_cols {
            if puzzle[r][c] == *"0" && count > 0 {
                header.push(count);
                count = 0;
            } else if puzzle[r][c] == *"1" {
                count += 1;
            }
        }
        if count > 0 || header.is_empty() {
            header.push(count);
        }

        row_headers.push(header);
    }

    Ok(())
}

fn iterate(
    grid: &mut Vec<Vec<&str>>,
    num_cols: usize,
    num_rows: usize,
    col_headers: &Vec<Vec<usize>>,
    row_headers: &Vec<Vec<usize>>,
) -> anyhow::Result<()> {
    let mut changed: bool;
    loop {
        changed = false;
        for r in 0..num_rows {
            if row_update(r, grid, num_cols, num_rows, col_headers, row_headers)? {
                changed = true;
            }
        }
        for c in 0..num_cols {
            if col_update(c, grid, num_cols, num_rows, col_headers, row_headers)? {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    Ok(())
}

fn row_update(
    row: usize,
    grid: &mut Vec<Vec<&str>>,
    num_cols: usize,
    num_rows: usize,
    col_headers: &Vec<Vec<usize>>,
    row_headers: &Vec<Vec<usize>>,
) -> anyhow::Result<bool> {
    let mut changed = false;

    let line: Vec<&str> = grid[row].clone();

    let new_line: Vec<&str> = solve_line(line.clone(), row_headers[row].clone())?;
    let mut cols_next: Vec<bool> = Vec::new();
    for c in 0..num_cols {
        if new_line[c] != line[c] {
            if line[c] == "2" {
                grid[row][c] = new_line[c];
                changed = true;
                cols_next.push(true);
                continue;
            } else {
                anyhow::bail!("Contradiction found");
            }
        }
        cols_next.push(false);
    }
    for c in 0..num_cols {
        if cols_next[c] {
            col_update(c, grid, num_cols, num_rows, col_headers, row_headers)?;
        }
    }

    Ok(changed)
}

fn col_update(
    col: usize,
    grid: &mut Vec<Vec<&str>>,
    num_cols: usize,
    num_rows: usize,
    col_headers: &Vec<Vec<usize>>,
    row_headers: &Vec<Vec<usize>>,
) -> anyhow::Result<bool> {
    let mut changed = false;

    let mut line: Vec<&str> = Vec::new();
    for r in 0..num_rows {
        line.push(grid[r][col]);
    }

    let new_line: Vec<&str> = solve_line(line.clone(), col_headers[col].clone())?;
    let mut rows_next: Vec<bool> = Vec::new();
    for r in 0..num_rows {
        if new_line[r] != line[r] {
            if line[r] == "2" {
                grid[r][col] = new_line[r];
                changed = true;
                rows_next.push(true);
                continue;
            } else {
                anyhow::bail!("Contradiction found");
            }
        }
        rows_next.push(false);
    }
    for r in 0..num_rows {
        if rows_next[r] {
            row_update(r, grid, num_cols, num_rows, col_headers, row_headers)?;
        }
    }

    Ok(changed)
}

fn solve_line(line: Vec<&str>, nums: Vec<usize>) -> anyhow::Result<Vec<&str>> {
    let mut new_line: Vec<&str> = Vec::new();
    if nums[0] == 0 {
        for _n in 0..line.len() {
            new_line.push("0");
        }
        return Ok(new_line);
    }

    let left: Vec<usize> = get_extreme(line.clone(), nums.clone(), 1)?;
    let right: Vec<usize> = get_extreme(line.clone(), nums.clone(), -1)?;
    let len: usize = line.len();
    let nums_len: usize = nums.len();
    let mut possiblities: Vec<Square> = Vec::new();
    for _n in 0..len {
        possiblities.push(Possibility::new());
    }

    //what we know from original info
    for i in 0..len {
        if line[i] == "1" {
            possiblities[i].already_filled();
        } else if line[i] == "0" {
            possiblities[i].already_empty();
        }
    }

    //edges outside the extremes
    for i in 0..left[0] {
        possiblities[i].cant_be_filled();
        possiblities[i].may_be_empty()?;
    }
    for i in right[nums_len - 1] + 1..len {
        possiblities[i].cant_be_filled();
        possiblities[i].may_be_empty()?;
    }
    //add possibilities to cells based on possible locations
    for num_index in 0..nums_len {
        let num = nums[num_index];
        if left[num_index] + num - 1 < right[num_index] - num + 1 {
            //min and max are disjoint
            //min is ambiguous
            for i in left[num_index]..left[num_index] + num {
                if line[i] == "2" {
                    possiblities[i].may_be_filled()?;
                    possiblities[i].may_be_empty()?;
                }
            }
            //in between, mark cbfilled where there's room and cbempty where it's not filled
            for index in left[num_index] + num..right[num_index] - num + 1 {
                if line[index] == "2" {
                    if theres_room(line.clone(), index as i8, num as i8, 1) {
                        for i in index..index + num {
                            possiblities[i].may_be_filled()?;
                        }
                    } else if theres_room(line.clone(), index as i8, num as i8, -1) {
                        for i in index..index + num {
                            possiblities[i].may_be_filled()?;
                        }
                    }
                    possiblities[index].may_be_empty()?;
                }
            }
            //max is ambiguous
            for i in right[num_index] - num + 1..right[num_index] + 1 {
                if line[i] == "2" {
                    possiblities[i].may_be_filled()?;
                    possiblities[i].may_be_empty()?;
                }
            }
        } else {
            //min and max overlap
            //write it all as ambiguous
            for i in left[num_index]..right[num_index] + 1 {
                if line[i] == "2" {
                    possiblities[i].may_be_filled()?;
                    possiblities[i].may_be_empty()?;
                }
            }
            //rewrite the overlapping portion
            for i in right[num_index] - num + 1..left[num_index] + num {
                if line[i] == "2" {
                    possiblities[i].cant_be_empty();
                }
            }
        }
    }
    for num_index in 1..nums_len {
        //if possible spots for consecutive nums have them non-overlapping
        for i in right[num_index - 1] + 1..left[num_index] {
            possiblities[i].cant_be_filled();
            possiblities[i].may_be_empty()?;
        }
    }

    //return new info
    for i in 0..len {
        let cb_filled = possiblities[i].could_be_filled;
        let cb_empty = possiblities[i].could_be_empty;
        if cb_filled == IsFilled::Yes && cb_empty == IsFilled::Yes {
            //we don't know whic yet
            new_line.push("2");
        } else if cb_filled == IsFilled::Yes {
            //confirm fill whether Empty is deconfirmed or uninitialized
            new_line.push("1");
        } else if cb_empty == IsFilled::Yes {
            //confirm empty whether Filled is deconfirmed or uninitialized
            new_line.push("0");
        } else if cb_filled == IsFilled::No && cb_empty == IsFilled::No {
            //if both are deconfirmed then there is a contradiction in the logic
            anyhow::bail!("Contradiction found");
        } else if cb_filled == IsFilled::No {
            //deconfirmed Filled, while Empty uninitialized, can say empty
            new_line.push("0");
        } else if cb_empty == IsFilled::No {
            //deconfirmed Empty, while Filled uninitialized, can say empty
            new_line.push("1");
        }
    }

    Ok(new_line)
}

fn get_extreme(line: Vec<&str>, nums: Vec<usize>, dir: i8) -> anyhow::Result<Vec<usize>> {
    let len: i8 = line.len() as i8;
    let nums_len: i8 = nums.len() as i8;

    //set up start positions
    let mut index: i8 = 0;
    let mut num_index: i8 = 0;
    let mut extreme: Vec<usize> = Vec::new();
    for _n in 0..nums_len as usize {
        extreme.push(0);
    }
    if dir == -1 {
        index = len - 1;
        num_index = nums_len - 1;
    }

    while num_index >= 0 && num_index < nums_len {
        let num: i8 = nums[num_index as usize] as i8;
        //check that there's room
        while !theres_room(line.clone(), index, num, dir) {
            index += dir;
        }
        //take down left/right index of the num-block location
        extreme[num_index as usize] = index as usize;
        //update indeces
        num_index += dir;
        index += (num + 1) * dir;
        if index < 0 || index > len {
            index -= dir;
        }
    }
    //keep going to end
    while index >= 0 && index < len {
        index += dir;
    }
    //go backwards and shift blocks as needed to cover
    //any existing filled Squares
    index -= dir;
    num_index -= dir;
    while num_index >= 0 && num_index < nums_len {
        let num: i8 = nums[num_index as usize] as i8;
        //find a spot where the Possibility doesn't cover an
        //existing filled cell
        let mut i = index;
        while i != extreme[num_index as usize] as i8 + ((num - 1) * dir) {
            if line[i as usize] == "1" {
                //may need to backtrack to cover the spot
                while !theres_room(line.clone(), i, num, -dir) {
                    i += dir;
                }
                extreme[num_index as usize] = (i + ((num - 1) * -dir)) as usize;
                break;
            }
            i -= dir;
        }
        index = extreme[num_index as usize] as i8 - dir;
        num_index -= dir;
    }
    //check that we did not move something off of an
    //existing colored in cell
    while index >= 0 && index < len {
        if line[index as usize] == "1" {
            anyhow::bail!("Contradiction found");
        }
        index -= dir;
    }

    Ok(extreme)
}

fn theres_room(line: Vec<&str>, index: i8, num: i8, dir: i8) -> bool {
    let len: i8 = line.len() as i8;
    //is the prior space not filled
    if index - dir != -1 && index - dir != len {
        if line[(index - dir) as usize] == "1" {
            return false;
        }
    }
    //are all the required spaces not crossed?
    let mut i: i8 = index;
    while i != index + num * dir {
        if line[i as usize] == "0" {
            return false;
        }
        i += dir;
    }
    //is the space next after not filled?
    let next_after: i8 = index + num * dir;
    if next_after != -1 && next_after != len {
        if line[next_after as usize] == "1" {
            return false;
        }
    }

    true
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum IsFilled {
    Unknown,
    Yes,
    No,
}

struct Square {
    could_be_filled: IsFilled,
    could_be_empty: IsFilled,
}

trait Possibility {
    fn new() -> Self;
    fn already_filled(&mut self);
    fn already_empty(&mut self);
    fn may_be_filled(&mut self) -> anyhow::Result<()>;
    fn may_be_empty(&mut self) -> anyhow::Result<()>;
    fn cant_be_filled(&mut self);
    fn cant_be_empty(&mut self);
}

impl Possibility for Square {
    fn new() -> Self {
        Self {
            could_be_filled: IsFilled::Unknown,
            could_be_empty: IsFilled::Unknown,
        }
    }

    fn already_filled(&mut self) {
        self.could_be_filled = IsFilled::Yes;
        self.could_be_empty = IsFilled::No;
    }

    fn already_empty(&mut self) {
        self.could_be_filled = IsFilled::No;
        self.could_be_empty = IsFilled::Yes;
    }

    fn may_be_filled(&mut self) -> anyhow::Result<()> {
        if self.could_be_filled == IsFilled::No {
            anyhow::bail!("Contradiction found");
        } else {
            self.could_be_filled = IsFilled::Yes;
        }

        Ok(())
    }

    fn may_be_empty(&mut self) -> anyhow::Result<()> {
        if self.could_be_empty == IsFilled::No {
            anyhow::bail!("Contradiction found");
        } else {
            self.could_be_empty = IsFilled::Yes;
        }

        Ok(())
    }

    fn cant_be_filled(&mut self) {
        self.could_be_filled = IsFilled::No;
    }

    fn cant_be_empty(&mut self) {
        self.could_be_empty = IsFilled::No;
    }
}
