mod db;
mod models;
mod ui;
mod io;

use crate::models::*;

fn main() {
    let x = Epic::new(String::from("foo"), String::from("bar"));
}
