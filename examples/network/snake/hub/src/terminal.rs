use crate::core::Core;
use capybara::instant::Instant;
use capybara::network::client::ConnectionStatus;

pub fn process(command: &str, core: &mut Core) {
    let tokens = command.split_whitespace().collect::<Vec<&str>>();

    match tokens.first() {
        Some(&"config") => process_config(&tokens, core),
        Some(&"clients") => process_clients(&tokens, core),
        Some(&"help") => process_help(&tokens, core),
        Some(&"workers") => process_workers(&tokens, core),
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
    let mut data = Vec::default();
    let config = core.config.read();

    data.push(format!(" - endpoint: {}", config.data.endpoint));
    data.push(format!(" - lobby tick: {} ms", config.data.lobby_tick));
    data.push(format!(" - worker status interval: {} ms", config.data.worker_status_interval));
    data.push(format!(" - packet delay base: {} ms", config.data.packet_delay_base));
    data.push(format!(" - packet delay variation: {} ms", config.data.packet_delay_variation));
    data.push(" - workers:".to_string());

    for worker in &core.config.read().data.workers {
        data.push(format!(
            "   - {} {} ({}), {}, {}",
            worker.id,
            worker.name,
            worker.flag,
            worker.address,
            if worker.enabled { "enabled " } else { "disabled " }
        ));
    }

    drop(config);

    println!("Current configuration:");
    println!("{}", data.join("\n"));
}

fn process_config_reload(_tokens: &[&str], core: &mut Core) {
    println!("Reloading configuration file");
    core.config.write().reload();
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
    let mut data = Vec::default();
    let clients = core.clients.read();

    for client in clients.iter() {
        let id = client.1.id;
        let address = client.1.address;
        let ping = client.1.ping.read();
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
    println!(" workers status - display a status of all workers");
}

fn process_workers(tokens: &[&str], core: &Core) {
    if tokens.len() < 2 {
        println!("Unknown parameter");
        return;
    }

    match tokens.get(1) {
        Some(&"status") => process_workers_status(tokens, core),
        _ => println!("Unknown parameter"),
    }
}

fn process_workers_status(_tokens: &[&str], core: &Core) {
    let mut output = Vec::default();
    let manager = core.workers.read();

    for worker in &manager.workers {
        let enabled = if worker.definition.enabled { "enabled" } else { "disabled" };
        if *worker.websocket.status.read() == ConnectionStatus::Connected {
            let ping = *worker.websocket.ping.read();
            output.push(format!(
                "{}, {} ({}) - {}, connected, ping {} ms",
                worker.definition.id, worker.definition.name, worker.definition.flag, enabled, ping
            ));
        } else {
            output.push(format!("{}, {} ({}) - {}, disconnected", worker.definition.id, worker.definition.name, worker.definition.flag, enabled));
        }
    }

    println!("Workers status:");

    for line in output {
        println!(" - {}", line);
    }
}
