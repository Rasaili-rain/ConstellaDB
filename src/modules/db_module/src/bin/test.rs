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

  engine.create_table(&user_table).unwrap();

  for i in 0..10 {
    let user_1 = Entity {
      of: "user_table".to_string(),
      data: vec![
        Data { name: "id".to_string(), value: Value::Int(i) },
        Data { name: "name".to_string(), value: Value::VarChar("abcd".to_string()) },
        Data { name: "password".to_string(), value: Value::VarChar("4567".to_string()) },
      ]
    };
    engine.insert(&user_1).unwrap();
  }

  // id == 1 && name == abcd
  let cond = Condition::And(
    Box::new(
      Condition::Compare {
        attr: "id".to_string(),
        value: Value::Int(1),
        op: Operator::Eq,
      }
    ),
    Box::new(
      Condition::Compare {
        attr: "name".to_string(),
        value: Value::VarChar("abcd".to_string()),
        op: Operator::Eq,
      }
    ),
  );

  let result = engine.select("user_table", vec!["id", "name", "password"], vec![cond]).unwrap();
  println!("{:?}", result);
}
