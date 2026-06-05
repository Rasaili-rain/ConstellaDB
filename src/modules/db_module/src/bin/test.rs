use db_module::*;

fn main() {
  let mut engine = Engine::new();

  let user = Table {
    name: "user_table".to_string(),
    attrs: vec![
      Attr { name: "id".to_string(), data_type: Type::Int },
      Attr { name: "name".to_string(), data_type: Type::VarChar(100) },
      Attr { name: "password".to_string(), data_type: Type::VarChar(8) },
    ]
  };

  engine.create_table(&user).unwrap();
}
