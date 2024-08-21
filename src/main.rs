use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

struct CommandSpec {
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub envs: HashMap<String, String>,
}

impl CommandSpec {
    fn parse(spec: &[String]) -> Option<CommandSpec> {
        let (label, spec) = match spec {
            [label, spec @ ..] if label.ends_with('/') && !label.contains('=') => {
                (Some(label.trim_end_matches('/').to_string()), spec)
            }
            spec => (None, spec),
        };

        let envs = spec
            .iter()
            .take_while(|item| item.contains('='))
            .map(|item| {
                let (k, v) = item.split_once('=').unwrap_or_else(Default::default);
                (k.to_string(), v.to_string())
            })
            .collect::<HashMap<_, _>>();

        let (command, args) = match &spec[envs.len()..] {
            [command, args @ ..] => (
                command.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
            ),
            _ => return None,
        };

        let label = label.unwrap_or_else(|| command.clone());

        Some(Self {
            label,
            command,
            args,
            envs,
        })
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some(separator) = args.first() else {
        usage(1);
    };

    if separator == "--help" || separator == "-h" {
        usage(0);
    }

    let commands: Vec<CommandSpec> = args[1..]
        .split(|arg| arg == separator)
        .filter_map(CommandSpec::parse)
        .collect();

    if commands.is_empty() {
        usage(1);
    }

    let max_label_len = commands
        .iter()
        .map(|spec| spec.label.len())
        .max()
        .unwrap_or(0);

    let mut handles = vec![];
    let failed = Arc::new(AtomicBool::new(false));

    for spec in commands {
        let failed = failed.clone();

        let handle = thread::spawn(move || -> std::io::Result<()> {
            let mut command = Command::new(&spec.command);
            command.args(&spec.args);
            command.envs(&spec.envs);
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(err) => {
                    println!(
                        "{:<width$}  = failed to start: {}",
                        spec.label,
                        err,
                        width = max_label_len
                    );
                    return Ok(());
                }
            };

            let stdout = child.stdout.take().expect("stdout must be captured");
            let stderr = child.stderr.take().expect("stderr must be captured");

            let label = spec.label.clone();

            let stdout_handle = thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    println!("{:<width$}  | {}", label, line, width = max_label_len);
                }
            });

            let label = spec.label.clone();

            let stderr_handle = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    println!("{:<width$} !| {}", label, line, width = max_label_len);
                }
            });

            let exit_status = child.wait()?;

            println!(
                "{:<width$}  = {}",
                spec.label,
                exit_status,
                width = max_label_len
            );

            if !exit_status.success() {
                failed.store(true, Ordering::Relaxed);
            }

            stdout_handle.join().unwrap();
            stderr_handle.join().unwrap();

            Ok(())
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    if failed.load(Ordering::Relaxed) {
        exit(1);
    }

    Ok(())
}

fn usage(exit_code: i32) -> ! {
    println!(
        "usage: paraexec ( <separator> [<label>/] [<ENV>=<value>...] <command> [<argument>...] )+"
    );
    exit(exit_code);
}
