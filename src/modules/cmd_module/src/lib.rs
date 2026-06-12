use std::io::{self, Write};
use db_module::*;

pub fn run() {
    let mut engine = Engine::new();

    loop {
        println!();
        println!("=== ConstellaDB Demo ===");
        println!("[1]  CREATE TABLE users");
        println!("[2]  INSERT 3 users");
        println!("[3]  SELECT all");
        println!("[4]  SELECT WHERE id = 2");
        println!("[5]  SELECT WHERE id > 1 AND id < 4");
        println!("[6]  SELECT WHERE id = 1 OR id = 3");
        println!("[7]  UPDATE name WHERE id = 2");
        println!("[8]  DELETE WHERE id = 1");
        println!("[9]  DROP TABLE users");
        println!("[0]  Exit");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => cmd_create_table(&mut engine),
            "2" => cmd_insert(&mut engine),
            "3" => cmd_select_all(&mut engine),
            "4" => cmd_select_by_id(&mut engine),
            "5" => cmd_select_and(&mut engine),
            "6" => cmd_select_or(&mut engine),
            "7" => cmd_update(&mut engine),
            "8" => cmd_delete(&mut engine),
            "9" => cmd_drop_table(&mut engine),
            "0" => { println!("Bye."); break; }
            _   => println!("Unknown command."),
        }
    }
}

fn cmd_create_table(engine: &mut Engine) {
    let table = Table {
        name: "users".to_string(),
        attrs: vec![
            Attr { name: "id".to_string(),   data_type: Type::Int },
            Attr { name: "name".to_string(),  data_type: Type::VarChar(50) },
            Attr { name: "email".to_string(), data_type: Type::VarChar(100) },
        ],
    };
    match engine.create_table(&table) {
        Ok(_)  => println!("Created table 'users'."),
        Err(e) => println!("Error: {}", e),
    }
}

fn cmd_insert(engine: &mut Engine) {
    let rows = vec![
        ("1", "alice", "alice@example.com"),
        ("2", "bob",   "bob@example.com"),
        ("3", "carol", "carol@example.com"),
    ];
    for (id, name, email) in rows {
        let entity = Entity {
            of: "users".to_string(),
            data: vec![
                Data { name: "id".to_string(),    value: Value::Int(id.parse().unwrap()) },
                Data { name: "name".to_string(),  value: Value::VarChar(name.to_string()) },
                Data { name: "email".to_string(), value: Value::VarChar(email.to_string()) },
            ],
        };
        match engine.insert(&entity) {
            Ok(_)  => println!("Inserted: id={} name={}", id, name),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn cmd_select_all(engine: &mut Engine) {
    print_result(engine.select("users", vec!["id", "name", "email"], vec![]));
}

fn cmd_select_by_id(engine: &mut Engine) {
    println!("SELECT WHERE id = 2");
    print_result(engine.select(
        "users",
        vec!["id", "name"],
        vec![Condition::Compare {
            attr:  "id".to_string(),
            value: Value::Int(2),
            op:    Operator::Eq,
        }],
    ));
}

fn cmd_select_and(engine: &mut Engine) {
    println!("SELECT WHERE id > 1 AND id < 4");
    print_result(engine.select(
        "users",
        vec!["id", "name"],
        vec![Condition::And(
            Box::new(Condition::Compare { attr: "id".to_string(), value: Value::Int(1), op: Operator::Gt }),
            Box::new(Condition::Compare { attr: "id".to_string(), value: Value::Int(4), op: Operator::Lt }),
        )],
    ));
}

fn cmd_select_or(engine: &mut Engine) {
    println!("SELECT WHERE id = 1 OR id = 3");
    print_result(engine.select(
        "users",
        vec!["id", "name"],
        vec![Condition::Or(
            Box::new(Condition::Compare { attr: "id".to_string(), value: Value::Int(1), op: Operator::Eq }),
            Box::new(Condition::Compare { attr: "id".to_string(), value: Value::Int(3), op: Operator::Eq }),
        )],
    ));
}

fn cmd_update(engine: &mut Engine) {
    println!("UPDATE name = 'bob_updated' WHERE id = 2");
    match engine.update(
        "users",
        vec![Data { name: "name".to_string(), value: Value::VarChar("bob_updated".to_string()) }],
        vec![Condition::Compare { attr: "id".to_string(), value: Value::Int(2), op: Operator::Eq }],
    ) {
        Ok(n)  => println!("Updated {} row(s).", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn cmd_delete(engine: &mut Engine) {
    println!("DELETE WHERE id = 1");
    match engine.delete(
        "users",
        vec![Condition::Compare { attr: "id".to_string(), value: Value::Int(1), op: Operator::Eq }],
    ) {
        Ok(n)  => println!("Deleted {} row(s).", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn cmd_drop_table(engine: &mut Engine) {
    match engine.drop_table("users") {
        Ok(_)  => println!("Dropped table 'users'."),
        Err(e) => println!("Error: {}", e),
    }
}

fn print_result(result: Result<Vec<Entity>, String>) {
    match result {
        Err(e)   => println!("Error: {}", e),
        Ok(rows) => {
            if rows.is_empty() {
                println!("(no rows)");
            } else {
                for row in &rows {
                    let parts: Vec<String> = row.data.iter()
                        .map(|d| format!("{}: {:?}", d.name, d.value))
                        .collect();
                    println!("  {{ {} }}", parts.join(", "));
                }
                println!("{} row(s).", rows.len());
            }
        }
    }
}