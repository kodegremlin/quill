use crate::row::Row;

mod color;
mod errors;
mod lang;
mod row;

fn main() {
    let row = Row::new("char with \t tab. \twith\t ¬˚˙¬˚∆ƒß");
    match row {
        Err(err) => println!("erorr: {}", err),
        Ok(result) => println!("{:?}", result),
    }
}
