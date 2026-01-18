use std::io::{self, Write};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::Parser;
use crossbeam_channel::unbounded;

use easypassword::core::GenerationMode;
use easypassword::{
    generate_password, start_keyboard_listener, Config, TextInjector, TriggerEvent,
};

#[derive(Parser)]
#[command(name = "easypassword")]
#[command(about = "A local-only deterministic password generator")]
#[command(version)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

fn wait_for_enter() {
    println!("\nPress Enter to exit...");
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn run(cli: Cli) -> anyhow::Result<()> {
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    println!("=== EasyPassword ===\n");

    let config_path = Config::config_path()?;
    println!("Config file: {:?}", config_path);

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            println!("\nERROR: Failed to load config: {}", e);
            println!("\nPlease create config file at: {:?}", config_path);
            println!("\nExample config.toml:");
            println!("  [default]");
            println!("  master_key = \"your_secret_key\"");
            println!("  trigger_prefix = \";;\"");
            return Err(e.into());
        }
    };

    let master_key = match &config.default.master_key {
        Some(key) if !key.is_empty() => {
            println!("Master key: [loaded from config]");
            key.clone()
        }
        _ => {
            println!("\nERROR: master_key not set in config!");
            println!("\nPlease add to {:?}:", config_path);
            println!("\n  [default]");
            println!("  master_key = \"your_secret_key_here\"");
            return Err(anyhow::anyhow!("master_key not configured"));
        }
    };

    println!("Trigger prefix: \"{}\"", config.default.trigger_prefix);
    println!(
        "Concat prefix: \"{}\"",
        config.default.concat_trigger_prefix
    );
    println!("\n--- Ready ---");
    println!(
        "Type: {}site.com<SPACE> to generate password (Argon2id)",
        config.default.trigger_prefix
    );
    println!(
        "Type: {}site.com<SPACE> to generate password (Concatenation)",
        config.default.concat_trigger_prefix
    );
    println!("Example: {}github.com ", config.default.trigger_prefix);
    println!("\nPress Ctrl+C to exit\n");

    let injection_active = Arc::new(AtomicBool::new(false));
    let (tx, rx) = unbounded::<TriggerEvent>();

    let triggers = vec![
        (
            config.default.trigger_prefix.clone(),
            GenerationMode::Argon2id,
        ),
        (
            config.default.concat_trigger_prefix.clone(),
            GenerationMode::Concatenation,
        ),
    ];

    let _listener_handle = start_keyboard_listener(tx, triggers, injection_active.clone())?;

    let mut injector = TextInjector::new(injection_active)?;

    println!("Listening for keyboard input...\n");

    loop {
        match rx.recv() {
            Ok(trigger) => {
                println!(
                    "[TRIGGER] Site: {} | Mode: {:?}",
                    trigger.site, trigger.mode
                );

                let mut password_config = config.get_password_config(&trigger.site);

                // Override mode based on trigger
                if trigger.mode == GenerationMode::Concatenation {
                    password_config.mode = GenerationMode::Concatenation;
                }

                let counter = config.get_counter(&trigger.site);

                match generate_password(&master_key, &trigger.site, counter, &password_config) {
                    Ok(password) => {
                        println!(
                            "[OK] Generated {} chars for {}",
                            password.len(),
                            trigger.site
                        );
                        match injector.replace_trigger(trigger.trigger_len, &password) {
                            Ok(_) => println!("[OK] Password injected"),
                            Err(e) => println!("[ERROR] Injection failed: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("[ERROR] Generation failed: {}", e);
                    }
                }
                println!();
            }
            Err(e) => {
                println!("[ERROR] Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        println!("\n[FATAL] {}", e);
        wait_for_enter();
        std::process::exit(1);
    }
}
