mod db;
mod models;
mod ui;
mod io;
mod nav;

use std::rc::Rc;

use db::JiraDatabase;
use nav::Navigator;

use crate::models::*;

fn main() {
    let db = Rc::new(JiraDatabase::new("./data/database.json"));
    let nvigator = Navigator::new(Rc::clone(&db));
}
