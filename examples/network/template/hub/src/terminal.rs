use crate::core::Core;

pub fn process(command: &str, core: &Core) {
    let tokens = command.split_whitespace().collect::<Vec<&str>>();

    match tokens.first() {
        Some(&"clients") => process_clients(&tokens, core),
        Some(&"help") => process_help(&tokens, core),
        _ => println!("Unknown command"),
    }
}

fn process_clients(tokens: &[&str], core: &Core) {
    if tokens.len() < 2 {
        println!("Unknown parameter");
        return;
    }

    match tokens.get(1) {
        Some(&"list") => process_clients_list(tokens, core),
        _ => println!("Unknown parameter"),
    }
}

fn process_clients_list(_tokens: &[&str], core: &Core) {
    let mut data = Vec::new();
    let clients = core.clients.read().unwrap();

    for client in clients.iter() {
        data.push(format!("{} - ping {}", client.1.id, client.1.ping.read().unwrap()))
    }

    drop(clients);

    println!("Conected clients:");
    println!("{}", data.join("\n"));
}

fn process_help(_tokens: &[&str], _core: &Core) {
    println!("Commands:");
    println!(" clients list - display a list of all connected clients");
    println!(" help - list all available commands");
}
