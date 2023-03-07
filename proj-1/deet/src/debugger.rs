use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use rustyline::history::FileHistory;

#[derive(Clone)]
pub struct Breakpoint {
    pub addr: usize,
    pub orig_byte: u8,
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<(), FileHistory>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, Option<Breakpoint>>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // Load the target executable file to initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<(), FileHistory>::new().expect("Create Editor fail");
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Breakpoint(bp_target) => {
                    let addr: usize;
                    if bp_target.starts_with("*") {
                        addr = Self::parse_address(&bp_target[1..]).unwrap();
                    } else if let Ok(line_number) = bp_target.parse::<usize>() {
                        if let Some(address) = self.debug_data.get_addr_for_line(None, line_number)
                        {
                            addr = address;
                        } else {
                            println!("line number can't find the corresponding address");
                            continue;
                        }
                    } else if let Some(address) =
                        self.debug_data.get_addr_for_function(None, &bp_target)
                    {
                        addr = address;
                    } else {
                        println!("{} can't be parsed to a breakpoint target", bp_target);
                        println!("Usage: b|break|breakpoint *address|line|func");
                        continue;
                    }

                    println!("Set breakpoint {} at {}", self.breakpoints.len(), addr);
                    // If there exits inferior, we should get orig_byte of new breakpoints
                    // and insert HashMap
                    if let Some(inferior) = &mut self.inferior {
                        match inferior.write_byte(addr, 0xcc) {
                            Ok(orig_byte) => {
                                self.breakpoints
                                    .insert(addr, Some(Breakpoint { addr, orig_byte }));
                            }
                            Err(err) => {
                                println!("Debugger::new breakpoint write_byte: {}", err)
                            }
                        }
                    } else {
                        self.breakpoints.insert(addr, None);
                    }
                }
                DebuggerCommand::Backtrace => {
                    if let Some(inferior) = &self.inferior {
                        if let Err(err) = inferior.print_backtrace(&self.debug_data) {
                            println!("Err print_backtrace: {}", err);
                        }
                    }
                }
                DebuggerCommand::Continue => {
                    if let Some(_) = &self.inferior {
                        self.inferior_continue_exec(&mut self.breakpoints.clone());
                    } else {
                        // continue when there is no inferior
                        println!("There is no inferior running");
                    }
                }
                DebuggerCommand::Run(args) => {
                    // If type run when there exists inferior, kill the child process.
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill().expect("inferior.kill wasn't running");
                    }
                    if let Some(inferior) =
                        Inferior::new(&self.target, &args, &mut self.breakpoints)
                    {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // milestone 1: make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        self.inferior_continue_exec(&mut self.breakpoints.clone());
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    // if there exists inferior, kill the child process
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill().expect("inferior.kill wasn't running");
                    }
                    return;
                }
            }
        }
    } // run

    /// parse a usize from a hexadecimal string
    fn parse_address(addr: &str) -> Option<usize> {
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        // println!("addr = {}", addr);
        usize::from_str_radix(addr_without_0x, 16).ok()
    }

    /// When continue command is typed, we should continue the inferior
    /// call inferior.continue_exec() and handle the Result
    fn inferior_continue_exec(&mut self, breakpoints: &mut HashMap<usize, Option<Breakpoint>>) {
        if let Some(inferior) = &mut self.inferior {
            match inferior.continue_exec(breakpoints) {
                Ok(status) => match status {
                    Status::Exited(exit_status_code) => {
                        self.inferior = None;
                        println!("Child exited (status {})", exit_status_code);
                    }
                    Status::Signaled(signal) => {
                        self.inferior = None;
                        println!("Child exited (signal {})", signal);
                    }
                    Status::Stopped(signal, rip) => {
                        println!("Child stopped (signal {})", signal);
                        if let Some(line) = self.debug_data.get_line_from_addr(rip) {
                            println!("Stopped at {}", line);
                        }
                    }
                },
                Err(err) => println!("Inferior can't be woken up and execute: {}", err),
            }
        } else {
            println!("inferior_continue_exec failed: there is no inferior");
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    let _ = self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
} // impl
