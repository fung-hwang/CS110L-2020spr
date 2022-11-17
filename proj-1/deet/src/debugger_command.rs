pub enum DebuggerCommand {
    Quit,
    Run(Vec<String>),
    Continue,
    Backtrace,
    Breakpoint(String),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "b" | "break" => Some(DebuggerCommand::Breakpoint(tokens[1].to_string())),
            "bt" | "back" | "backtrace" => Some(DebuggerCommand::Backtrace),
            "c" | "cont" | "continue" => Some(DebuggerCommand::Continue),
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            // Default case:
            _ => None,
        }
    }
}
