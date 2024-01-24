use crate::core::Core;
use capybara::instant::Instant;
use log::info;

pub fn process(command: &str, core: &mut Core) {
    let tokens = command.split_whitespace().collect::<Vec<&str>>();

    match tokens.first() {
        Some(&"config") => process_config(&tokens, core),
        Some(&"clients") => process_clients(&tokens, core),
        Some(&"help") => process_help(&tokens, core),
        _ => println!("Unknown command"),
    }
}

fn process_config(tokens: &[&str], core: &mut Core) {
    if tokens.len() < 2 {
        println!("Unknown parameter");
        return;
    }

    match tokens.get(1) {
        Some(&"show") => process_config_show(tokens, core),
        Some(&"reload") => process_config_reload(tokens, core),
        _ => println!("Unknown parameter"),
    }
}

fn process_config_show(_tokens: &[&str], core: &Core) {
    println!(" - tick: {} ms", core.config.data.tick);
    println!(" - servers:");

    for server in &core.config.data.servers {
        println!("   - {} ({}), {}, {}", server.name, server.flag, server.address, if server.enabled { "enabled " } else { "disabled " });
    }
}

fn process_config_reload(_tokens: &[&str], core: &mut Core) {
    println!("Reloading configuration file");
    core.config.reload();
    println!("Configuration reloaded");
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
        let id = client.1.id;
        let address = client.1.address;
        let ping = client.1.ping.read().unwrap();
        let online_time = (Instant::now() - client.1.join_time).as_secs() / 60;

        data.push(format!("{}, {}, ping {}, online {} minutes", id, address, ping, online_time));
    }

    drop(clients);

    println!("Conected clients:");
    println!("{}", data.join("\n"));
}

fn process_help(_tokens: &[&str], _core: &Core) {
    println!("Commands:");
    println!(" config show - show currently loaded configuration");
    println!(" config reload - reload configuration from the file");
    println!(" clients list - display a list of all connected clients");
    println!(" help - list all available commands");
}
