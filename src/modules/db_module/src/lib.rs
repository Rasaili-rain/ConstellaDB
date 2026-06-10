use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::result::Result;
use std::io::{BufWriter, BufReader, Read, Write};
use std::fmt;
use serde::{Deserialize, Serialize};


pub const DB_DIR: &str = "DB";
pub const SCHEMA_FILE: &str = "schemas.json";


pub enum Operator {
  Eq,    // ==
  Ne,    // !=
  Lt,    // <
  Le,    // <=
  Gt,    // >
  Ge,    // >=
}

pub enum Condition {
  Compare {
    attr: String,
    value: Value,
    op: Operator,
  },
  And(Box<Condition>, Box<Condition>),
  Or(Box<Condition>, Box<Condition>),
}


//=======
// Table
//=======

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
  Int,
  VarChar(usize),
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Type::Int => write!(f, "INT"),
      Type::VarChar(size) => write!(f, "VARCHAR({})", size),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attr {
  pub name: String,
  pub data_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
  pub name: String,
  pub attrs: Vec<Attr>,
}

impl Table {
  pub fn attr_exists(&self, attr_name: &str) -> bool {
    for attr in &self.attrs {
      if attr_name == attr.name {
        return true;
      }
    }
    return false;
  }
}


//=========
// Entity
//=========

#[derive(Debug, Clone)]
pub enum Value {
  Int(i32),
  VarChar(String),
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Int(_) => write!(f, "INT"),
      Value::VarChar(_) => write!(f, "VARCHAR"),
    }
  }
}

#[derive(Debug)]
pub struct Data {
  pub name: String,
  pub value: Value,
}

#[derive(Debug)]
pub struct Entity {
  pub of: String,
  pub data: Vec<Data>,
}


//=========================
// Engine of the db module
//=========================

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

  //==============================
  // DATABASE MODULE PUBLIC API'S
  //==============================

  pub fn create_table(&mut self, table: &Table) -> Result<(), String> {
    if self.table_exists(&table.name) {
      return Err(format!("Table with name '{}' already exists", table.name));
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

  pub fn insert(&mut self, entity: &Entity) -> Result<(), String> {
    let table = match self.get_table(&entity.of) {
      Some(t) => t,
      None    => return Err(format!("Table with name '{}' doesn't exists", entity.of)),
    };

    // Validate the data attributes
    self.validate_entity_data(table, entity)?;

    for data in &entity.data {
      let attr = table
        .attrs
        .iter()
        .find(|a| a.name == data.name)
        .unwrap();

      let path = PathBuf::from(DB_DIR)
        .join(&table.name)
        .join(format!("{}.col", &data.name));

      // Open the required attribute file
      let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.to_str().unwrap())
        .unwrap();

      // Store the byte
      match (&attr.data_type, &data.value) {
        (Type::Int, Value::Int(v)) => {
          file.write_all(&v.to_le_bytes())
            .map_err(|e| e.to_string())?;
        },

        (Type::VarChar(size), Value::VarChar(v)) => {
          // Resizing to the varchar size
          let mut bytes = v.as_bytes().to_vec();
          bytes.resize(*size, 0);

          file.write_all(&bytes)
            .map_err(|e| e.to_string())?;
        },

        _ => { return Err("Unreachable!".to_string()); }
      }
    }

    Ok(())
  }

  pub fn select(&mut self, table_name: &str, attrs: Vec<&str>, conditions: Vec<Condition>) -> Result<Vec<Entity>, String> {
    let table = match self.get_table(table_name) {
      Some(t) => t,
      None    => return Err(format!("Table with name '{}' doesn't exists", table_name)),
    };

    // Verify attributes
    for attr in &attrs {
      if !table.attr_exists(attr) {
        return Err(format!("Attribute '{}' doesn't exists in table {}", attr, table.name));
      }
    }

    // Collect the required attributes
    let req_attrs: Vec<&Attr> = table.attrs
      .iter()
      .filter(|a| attrs.contains(&a.name.as_str()))
      .collect();

    // Fetch the columns
    let mut columns: Vec<Vec<Value>> = Vec::new();
    for attr in &req_attrs {
      columns.push(self.load_column(&table, &attr)?);
    }

    // If no columns were fetched then return empty
    if columns.is_empty() {
      return Ok(Vec::new());
    }

    let row_count = columns[0].len();
    let mut result: Vec<Entity> = Vec::new();

    for row in 0..row_count {
      let mut row_data: Vec<Data> = Vec::new();

      // Create row data
      for (idx, attr) in req_attrs.iter().enumerate() {
        row_data.push(Data {
          name: attr.name.clone(),
          value: columns[idx][row].clone(),
        });
      }

      // Create entity
      let entity = Entity {
        of: table.name.clone(),
        data: row_data,
      };

      // Check the condition
      let matches = conditions
        .iter()
        .all(|c| self.match_condition(&entity, c));

      // If all condition passes then its the result
      if matches {
        result.push(entity);
      }
    }

    Ok(result)
  }


  //==============================
  // DATABASE MODULE PRIVATE API'S
  //==============================

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

  fn get_table(&self, name: &str) -> Option<&Table> {
    for table in self.tables.iter() {
      if table.name == name {
        return Some(table);
      }
    }
    None
  }

  fn validate_entity_data(&self, table: &Table, entity: &Entity) -> Result<(), String> {
    for attr in &table.attrs {
      // Checking if all the attributes are provided or not
      let data = entity
        .data
        .iter()
        .find(|d| d.name == attr.name)
        .ok_or_else(|| format!("Missing attribute '{}'", attr.name))?;

      // Typechecking the attributes
      match (&attr.data_type, &data.value) {
        (Type::Int,        Value::Int(_))     => {},
        (Type::VarChar(_), Value::VarChar(_)) => {},
        _ => {
          return Err(format!(
            "Type mismatch for '{}', required '{}' got '{}'",
            attr.name, attr.data_type, data.value
          ));
        }
      }
    }

    // Check for extra attributes
    for data in &entity.data {
      if !table.attrs.iter().any(|a| a.name == data.name) {
        return Err(format!(
          "Unknown attribute '{}'",
          data.name
        ));
      }
    }

    Ok(())
  }

  fn load_column(&self, table: &Table, attr: &Attr) -> Result<Vec<Value>, String> {
    if !self.table_exists(&table.name) {
      return Err(format!("Table with name '{}' doesn't exists", table.name));
    }

    if !table.attr_exists(&attr.name) {
      return Err(format!("Attribute '{}' doesn't exists in table {}", attr.name, table.name));
    }

    // Open the attribute file
    let path = PathBuf::from(DB_DIR)
      .join(&table.name)
      .join(format!("{}.col", &attr.name));

    let mut file = File::open(path).map_err(|e| e.to_string())?;

    // Final value array
    let mut values: Vec<Value> = Vec::new();

    match attr.data_type {
      Type::Int => {
        // Buffer size should be 4
        let mut buff = [0u8; 4];

        while file.read_exact(&mut buff).is_ok() {
          values.push(Value::Int(i32::from_le_bytes(buff)));
        }
      },

      Type::VarChar(size) => {
        // Buffer size should be size of the varchar size
        let mut buff = vec![0u8; size];

        while file.read_exact(&mut buff).is_ok() {
          let s = String::from_utf8_lossy(&buff)
            .trim_end_matches('\0')
            .to_string();

          values.push(Value::VarChar(s));
        }
      },
    };

    Ok(values)
  }

  fn match_condition(&self, entity: &Entity, condition: &Condition) -> bool {
    match condition {
      Condition::Compare { attr, value, op } => {
        // Find the attribute for the compare
        let Some(data) = entity.data
          .iter()
          .find(|d| d.name == *attr)
        else {
          return false;
        };

        match (&data.value, value, op) {
          (Value::Int(a), Value::Int(b), Operator::Eq) => a == b,
          (Value::Int(a), Value::Int(b), Operator::Ne) => a != b,
          (Value::Int(a), Value::Int(b), Operator::Gt) => a >  b,
          (Value::Int(a), Value::Int(b), Operator::Ge) => a >= b,
          (Value::Int(a), Value::Int(b), Operator::Lt) => a <  b,
          (Value::Int(a), Value::Int(b), Operator::Le) => a <= b,

          (Value::VarChar(a), Value::VarChar(b), Operator::Eq) => a == b,
          (Value::VarChar(a), Value::VarChar(b), Operator::Ne) => a != b,

          _ => false,
        }
      },

      Condition::And(left, right) => {
        self.match_condition(entity, left) && self.match_condition(entity, right)
      },

      Condition::Or(left, right) => {
        self.match_condition(entity, left) || self.match_condition(entity, right)
      },
    }
  }
}
