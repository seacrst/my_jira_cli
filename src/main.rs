mod db;
mod models;
mod ui;
mod io;
mod nav;

use std::rc::Rc;

use db::JiraDatabase;
use io::get_input;
use nav::Navigator;

use crate::models::*;

fn main() {
    let db = Rc::new(JiraDatabase::new("./data/database.json"));
    let mut navigator = Navigator::new(Rc::clone(&db));
    
    loop {
        clearscreen::clear().unwrap();

        match navigator.get_current_page() {
            Some(page) => {
                if let Err(error) = page.draw_page() {
                    println!(
                        "Error rendering page: {}\nPress any key to continue...",
                        error
                    );
                    io::wait_for_key_press();
                };                

                let action_r = page.handle_input(get_input().trim());
                
                match action_r {
                    Ok(action) => {
                        if let Some(action) = action {
                            if let Err(error) = navigator.handle_action(action) {
                                println!("Error handling processing user input: {}\nPress any key to continue...", error);
                                io::wait_for_key_press();
                            }
                        }
                    },
                    Err(err) => {
                        println!(
                            "Error getting user input: {}\nPress any key to continue...",
                            err
                        );
                        io::wait_for_key_press();
                    }
                }
            },
            _ => break
        }
        
    }
}
