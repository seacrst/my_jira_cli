pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed
}

pub struct Epic {
  pub name: String,
  pub description: String,
  pub status: Status
}

impl Epic {
  pub fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      status: Status::Open
    }
  }
}

pub struct Story {
  pub name: String,
  pub description: String,
  pub status: Status
}

impl Story {
  pub fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      status: Status::Open
    }
  }
}