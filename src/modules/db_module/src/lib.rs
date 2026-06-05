use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::result::Result;
use std::io::{BufWriter, BufReader};
use serde::{Deserialize, Serialize};


pub const DB_DIR: &str = "DB";
pub const SCHEMA_FILE: &str = "schemas.json";

#[derive(Clone, Serialize, Deserialize)]
pub enum Type {
  Int,
  VarChar(usize),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Attr {
  pub name: String,
  pub data_type: Type,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Table {
  pub name: String,
  pub attrs: Vec<Attr>,
}

pub struct Engine {
  tables: Vec<Table>,
}

impl Engine {
  pub fn new() -> Self {
    if !Path::new(DB_DIR).exists() {

      fs::create_dir(DB_DIR).unwrap();
      File::create(
        PathBuf::from(DB_DIR)
          .join(SCHEMA_FILE)
          .to_str()
          .unwrap()
      ).unwrap();

      return Self {
        tables: Vec::new(),
      };
    }

    return Self {
        tables: Self::load_schema(),
    }
  }

  pub fn create_table(&mut self, table: &Table) -> Result<(), String> {
    if self.table_exists(&table.name) {
      return Err(format!("Table with name {} already exists", table.name));
    }

    // Create the table dir
    let table_dir_path = PathBuf::from(DB_DIR)
      .join(&table.name);

    fs::create_dir_all(table_dir_path.to_str().unwrap()).unwrap();

    // Create attribute files
    for attr in table.attrs.iter() {
      let attr_file_path = table_dir_path
        .clone()
        .join(format!("{}.col", attr.name));

      File::create(attr_file_path.to_str().unwrap()).unwrap();
    }

    // Store the table
    self.tables.push(table.clone());

    // Save the schema
    self.save_schema();

    Ok(())
  }

  fn save_schema(&self) {
    let file = File::create(
      PathBuf::from(DB_DIR)
        .join(SCHEMA_FILE)
        .to_str()
        .unwrap()
    ).unwrap();

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &self.tables).unwrap();
  }

  fn load_schema() -> Vec<Table> {
    let file = File::open(
      PathBuf::from(DB_DIR)
        .join(SCHEMA_FILE)
        .to_str()
        .unwrap()
    ).unwrap();

    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
  }

  fn table_exists(&self, name: &str) -> bool {
    for table in self.tables.iter() {
      if table.name == name {
        return true;
      }
    }
    false
  }
}
