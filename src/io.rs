pub fn get_input() -> String {
  let mut s = String::new();

  std::io::stdin().read_line(&mut s).unwrap();

  s
}

pub fn wait_for_key_press() {
  std::io::stdin().read_line(&mut String::new()).unwrap();
}