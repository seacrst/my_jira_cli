use std::fs;

use anyhow::{anyhow, Ok, Result};

use crate::models::*;

pub trait Database {
  fn read(&self) -> Result<DbState>;
  fn write(&self, state: &DbState) -> Result<()>;
}

pub struct JiraDatabase {
  pub database: Box<dyn Database>
}

impl JiraDatabase {
  pub fn new(path: &str) -> Self {
    Self {
      database: Box::new(JsonFileDb::from(path))
    }
  }

  pub fn read(&self) -> Result<DbState> {
    let db = self.database.read().expect("Read JIRA error");
    Ok(db)
  }
}

impl JiraDatabase {
  pub fn create_epic(&self, epic: Epic) -> Result<u32> {
    let mut state = self.database.read()?;
    let id = state.last_item_id + 1;

    state.last_item_id = id;
    state.epics.insert(id, epic);

    self.database.write(&state)?;
    Ok(id)
  }

  pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
    let mut db = self.database.read()?;
    let stories = &db.epics
      .get(&epic_id)
      .ok_or_else(|| anyhow!("could not find epic in database!"))?
      .stories;

    for id in stories {
      db.stories.remove(id);
    }

    db.epics.remove(&epic_id);
    self.database.write(&db)?;
    Ok(())
  }

  pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
    let mut db = self.database.read()?;
    db.epics
      .get_mut(&epic_id)
      .ok_or_else(|| anyhow!("could not find epic in database!"))?
      .status = status;
    self.database.write(&db)?;
    Ok(())
  }
}

impl JiraDatabase {
  pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
    let mut state = self.database.read()?;
    let id = state.last_item_id + 1;

    state.last_item_id = id;
    state.stories.insert(id, story);
    state.epics
      .get_mut(&epic_id)
      .ok_or_else(|| anyhow!("could not find epic in database!"))?
      .stories.push(id);

    self.database.write(&state)?;
    Ok(id)
  }

  pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
    let mut state = self.database.read()?;

    let epic = state.epics
      .get_mut(&epic_id)
      .ok_or_else(|| anyhow!("could not find epic in database!"))?;

    let story_idx = epic.stories.iter()
      .position(|id| id == &story_id)
      .ok_or_else(|| anyhow!("story id not found in epic stories vector"))?;
    epic.stories.remove(story_idx);
    state.stories.remove(&story_id);

    self.database.write(&state)?;
    Ok(())
  }

  pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
    let mut db = self.database.read()?;
  
    db.stories
      .get_mut(&story_id)
      .ok_or_else(|| anyhow!("could not find epic in database!"))?
      .status = status;

    self.database.write(&db)?;
    Ok(())
  }
}

struct JsonFileDb {
  pub file_path: String
}

impl JsonFileDb {
  fn from(path: &str) -> Self {
    JsonFileDb {
      file_path: String::from(path)
    }
  }
}

impl Database for JsonFileDb {
  fn read(&self) ->  Result<DbState> {
      let db = fs::read_to_string(&self.file_path)?;
      let json: DbState = serde_json::from_str(&db)?;
      Ok(json)
  }

  fn write(&self, state: &DbState) -> Result<()> {
    let content = &serde_json::to_vec(state)?;
    fs::write(&self.file_path, content)?;
    Ok(())
  }
}

pub mod test_utils {
  use std::{cell::RefCell, collections::HashMap};

  use super::*;
  
  pub struct MockDB {
     pub last_written_state: RefCell<DbState>
  }

  impl MockDB {
      pub fn new() -> Self {
        Self { last_written_state: RefCell::new(DbState { last_item_id: 0, epics: HashMap::new(), stories: HashMap::new() }) }
      }    
  }

  impl Database for MockDB {
    fn read(&self) -> Result<DbState> {
        let state = self.last_written_state.borrow().clone();
        Ok(state)
    }

    fn write(&self, db_state: &DbState) -> Result<()> {
        let latest_state = &self.last_written_state;
        *latest_state.borrow_mut() = db_state.clone();
        Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use self::test_utils::MockDB;
  use super::*;

  #[test]
  fn create_epic_should_work() {
    let jira_db = JiraDatabase { database: Box::new(MockDB::new()) };
    let epic = Epic::new(String::default(), String::default());

    let r = jira_db.create_epic(epic.clone());

    assert_eq!(r.is_ok(), true);

    let epic_id = r.unwrap();
    let state = jira_db.read().unwrap();

    let expected_id = 1;

    assert_eq!(epic_id, expected_id);
    assert_eq!(state.last_item_id, expected_id);
    assert_eq!(state.epics.get(&epic_id), Some(&epic));
  }

  #[test]
  fn create_story_should_work() {
    let jira_db = JiraDatabase { database: Box::new(MockDB::new()) };
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());

    let r = jira_db.create_epic(epic);
    assert_eq!(r.is_ok(), true);

    let epic_id = r.unwrap();

    let r = jira_db.create_story(story.clone(), epic_id);
    assert_eq!(r.is_ok(), true);

    let id = r.unwrap();
    let db_state = jira_db.read().unwrap();

    let expected_id = 2;

    assert_eq!(id, expected_id);
    assert_eq!(db_state.last_item_id, expected_id);
    assert_eq!(db_state.epics.get(&epic_id).unwrap().stories.contains(&id), true);
    assert_eq!(db_state.stories.get(&id), Some(&story));
  }

  #[test]
  fn create_story_should_error_if_invalid_epic_id() {
    let db = JiraDatabase {
        database: Box::new(MockDB::new()),
    };
    let story = Story::new(String::default(), String::default());

    let non_existent_epic_id = 999;

    let result = db.create_story(story, non_existent_epic_id);
    assert_eq!(result.is_err(), true);
  }

  #[test]
  fn delete_story_should_error_if_invalid_epic_id() {
    let db = JiraDatabase { database: Box::new(MockDB::new()) };
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());
    let r = db.create_epic(epic);

    assert_eq!(r.is_ok(), true);

    let epic_id = r.unwrap();
    let r = db.create_story(story, epic_id);

    assert_eq!(r.is_ok(), true);

    let story_id = r.unwrap();
    let non_existent_epic_id = 999;
    let r = db.delete_story(non_existent_epic_id, story_id);

    assert_eq!(r.is_err(), true);
  }

  #[test]
  fn delete_epic_should_error_if_invalid_epic_id() {
    let jira_db = JiraDatabase { database: Box::new(MockDB::new()) };

    let non_existent_epic_id = 999;

    let r = jira_db.delete_epic(non_existent_epic_id);
    assert_eq!(r.is_err(), true);
  }

  #[test]
  fn delete_story_should_error_if_story_not_found_in_epic() {
    let jira_db = JiraDatabase { database: Box::new(MockDB::new()) };
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());

    let result = jira_db.create_epic(epic);
    assert_eq!(result.is_ok(), true);

    let epic_id = result.unwrap();

    let r = jira_db.create_story(story, epic_id);
    assert_eq!(r.is_ok(), true);

    let non_existent_story_id = 999;
    
    let r = jira_db.delete_story(epic_id, non_existent_story_id);
    assert_eq!(r.is_err(), true);
  }

  #[test]
  fn delete_story_should_work() {
    let db = JiraDatabase {
        database: Box::new(MockDB::new()),
    };
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());

    let result = db.create_epic(epic);
    assert_eq!(result.is_ok(), true);

    let epic_id = result.unwrap();

    let result = db.create_story(story, epic_id);
    assert_eq!(result.is_ok(), true);

    let story_id = result.unwrap();

    let result = db.delete_story(epic_id, story_id);
    assert_eq!(result.is_ok(), true);

    let db_state = db.read().unwrap();

    let expected_last_id = 2;

    assert_eq!(db_state.last_item_id, expected_last_id);
    assert_eq!(
        db_state
            .epics
            .get(&epic_id)
            .unwrap()
            .stories
            .contains(&story_id),
        false
    );
    assert_eq!(db_state.stories.get(&story_id), None);
  }

  #[test]
  fn delete_epic_should_work() {
    let db = JiraDatabase {
        database: Box::new(MockDB::new()),
    };
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());

    let result = db.create_epic(epic);
    assert_eq!(result.is_ok(), true);

    let epic_id = result.unwrap();

    let result = db.create_story(story, epic_id);
    assert_eq!(result.is_ok(), true);

    let story_id = result.unwrap();

    let result = db.delete_epic(epic_id);
    assert_eq!(result.is_ok(), true);

    let db_state = db.read().unwrap();

    let expected_last_id = 2;

    assert_eq!(db_state.last_item_id, expected_last_id);
    assert_eq!(db_state.epics.get(&epic_id), None);
    assert_eq!(db_state.stories.get(&story_id), None);
  }

  #[test]
  fn update_epic_status_should_work() {
    let db = JiraDatabase { database: Box::new(MockDB::new()) };
    let epic = Epic::new(String::default(), String::default());

    let r = db.create_epic(epic);

    assert_eq!(r.is_ok(), true);

    let epic_id = r.unwrap();

    let r = db.update_epic_status(epic_id, Status::Closed);

    assert_eq!(r.is_ok(), true);

    let db_state = db.read().unwrap();

    assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
  }

  #[test]
  fn update_story_status_should_work() {
    let db = JiraDatabase { database: Box::new(MockDB::new())};
    let epic = Epic::new(String::default(), String::default());
    let story = Story::new(String::default(), String::default());

    let r = db.create_epic(epic);
    let epic_id = r.unwrap();
    let r = db.create_story(story, epic_id);
    let story_id = r.unwrap();
    let r = db.update_story_status(story_id, Status::Closed);

    assert_eq!(r.is_ok(), true);

    let db_state = db.read().unwrap();

    assert_eq!(
      db_state.stories.get(&story_id).unwrap().status,
      Status::Closed
    );
  }

  #[test]
  fn update_epic_status_should_error_if_invalid_epic_id() {
    let db = JiraDatabase { database: Box::new(MockDB::new()) };
    let non_existent_epic_id = 999;

    let r = db.update_epic_status(non_existent_epic_id, Status::Closed);
    assert_eq!(r.is_err(), true);
  }

  #[test]
  fn update_story_status_should_error_if_invalid_story_id() {
    let db = JiraDatabase { database: Box::new(MockDB::new()) };
    let non_existent_story_id = 999;

    let result = db.update_story_status(non_existent_story_id, Status::Closed);
    assert_eq!(result.is_err(), true);
  }
  
  mod database {
    use std::collections::HashMap;
    use std::io::Write;
    use crate::{DbState, Epic, Story, db::Database};
    use super::JsonFileDb;

    #[test]
    fn read_fails_with_incorrect_path() {
      let json_db = JsonFileDb::from("not_exists");

      assert_eq!(json_db.read().is_ok(), false);
    }

    #[test]
    fn read_parse_fails() {
      let mut tf = tempfile::NamedTempFile::new().unwrap();
      let content = r#"{"last_item_id": 0, "epics": {},"stories}"#;
      write!(tf, "{content}").unwrap();

      let result = JsonFileDb::from(
        tf.path()
        .to_str()
        .expect("tempfile error")
      ).read();
      assert_eq!(result.is_err(), true);
    }

    #[test]
    fn read_parse_success() {
      let mut tf = tempfile::NamedTempFile::new().unwrap();
      let content = r#"{"last_item_id": 0, "epics": {},"stories": {}}"#;
      write!(tf, "{content}").unwrap();

      let result = JsonFileDb::from(tf.path().to_str().expect("tempfile error"))
        .read();
      assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn write_success() {
      let mut tf = tempfile::NamedTempFile::new().unwrap();

      let content = r#"{"last_item_id": 0, "epics": {},"stories": {}}"#;
      write!(tf, "{content}").unwrap();

      let db = JsonFileDb::from(tf.path().to_str().expect("tempfile error"));
      let story = Story::new(String::from("story name"), String::from("story description"));
      let epic = Epic::new(String::from("epic name"), String::from("epic description"));
      
      let mut stories = HashMap::new();
      let mut epics = HashMap::new();

      
      stories.insert(4, story);
      epics.insert(3, epic);

      let state = DbState { last_item_id: 3, epics, stories };

      let write_r = db.write(&state);
      let read_r = db.read().unwrap();

      assert_eq!(write_r.is_ok(), true);
      assert_eq!(read_r, state);
    }
  }
}