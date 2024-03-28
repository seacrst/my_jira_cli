use std::cmp::Ordering;
use ellipse::Ellipse;

pub fn get_column_string(text: &str, width: usize) -> String {
  let len = text.len();

  match len.cmp(&width) {
    Ordering::Equal => text.to_string(),
    Ordering::Less => {
      let left_over = width - len;
      let mut output_string = text.to_string();

      for _ in 0..left_over {
        output_string.push(' ');
      }

      output_string
    },
    Ordering::Greater => {
      if width == 0 {
        return String::default();
      } else if width == 1 {
        return String::from(".");
      } else if width == 2 {
        return String::from("..");
      } else if width == 3 {
        return String::from("...");
      }
      text.truncate_ellipse(width-3).to_string()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_column_string() {
    let text1 = "";
    let text2 = "test";
    let text3 = "testme";
    let text4 = "testmetest";

    let width = 0;

    assert_eq!(get_column_string(text4, width), "".to_owned());

    let width = 1;

    assert_eq!(get_column_string(text4, width), ".".to_owned());

    let width = 2;

    assert_eq!(get_column_string(text4, width), "..".to_owned());

    let width = 3;

    assert_eq!(get_column_string(text4, width), "...".to_owned());

    let width = 4;

    assert_eq!(get_column_string(text4, width), "t...".to_owned());

    let width = 6;

    assert_eq!(get_column_string(text1, width), "      ".to_owned());
    assert_eq!(get_column_string(text2, width), "test  ".to_owned());
    assert_eq!(get_column_string(text3, width), "testme".to_owned());
    assert_eq!(get_column_string(text4, width), "tes...".to_owned());
  } 
}