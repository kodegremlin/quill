use crate::row::Row;

mod errors;
mod row;

fn main() {
    let row = Row::new("char with \t tab. \twith\t ¬˚˙¬˚∆ƒß");
    match row {
        Err(err) => println!("erorr: {}", err),
        Ok(result) => println!("{:?}", result),
    }
}
