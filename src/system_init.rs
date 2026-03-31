use crate::commands::{built_in_command_names, get_commands};
use crate::setup::run_setup;
use crate::tools::get_tools;

pub fn build_system_init_message(trusted: bool) -> String {
    let setup = run_setup(None, trusted);
    let commands = get_commands(true, true);
    let tools = get_tools(false, true, None);
    let mut lines = vec![
        "# System Init".into(),
        String::new(),
        format!("Trusted: {}", setup.trusted),
        format!("Built-in command names: {}", built_in_command_names().len()),
        format!("Loaded command entries: {}", commands.len()),
        format!("Loaded tool entries: {}", tools.len()),
        String::new(),
        "Startup steps:".into(),
    ];
    for step in setup.setup.startup_steps() {
        lines.push(format!("- {}", step));
    }
    lines.join("\n")
}
