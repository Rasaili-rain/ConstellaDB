use db_module::*;

fn main() {
  let mut engine = Engine::new();

  let user_table = Table {
    name: "user_table".to_string(),
    attrs: vec![
      Attr { name: "id".to_string(), data_type: Type::Int },
      Attr { name: "name".to_string(), data_type: Type::VarChar(100) },
      Attr { name: "password".to_string(), data_type: Type::VarChar(8) },
    ]
  };

  //engine.create_table(&user_table).unwrap();

  let user_1 = Entity {
    of: "user_table".to_string(),
    data: vec![
      Data { name: "id".to_string(), value: Value::Int(123) },
      Data { name: "name".to_string(), value: Value::VarChar("user1".to_string()) },
      Data { name: "password".to_string(), value: Value::VarChar("123".to_string()) },
    ]
  };

  //engine.insert(&user_1).unwrap();

  let cond = Condition::Compare {
    attr: "id".to_string(),
    value: Value::Int(123),
    op: Operator::Eq,
  };

  //engine.select("user_table", vec!["id", "name"], Condition {

  //})
}
