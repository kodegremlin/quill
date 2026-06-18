use crate::row::Row;

mod buffer;
mod color;
mod diff;
mod history;
mod lang;
mod row;
mod terminal;

/* TODO 1. document the code and for simple functions just introduce what
the function does.
*/

fn main() {
    let row = Row::new("char with \t tab. \twith\t ¬˚˙¬˚∆ƒß");
    match row {
        Err(err) => println!("erorr: {}", err),
        Ok(result) => println!("{:?}", result),
    }
}
