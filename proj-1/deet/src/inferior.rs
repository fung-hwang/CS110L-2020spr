use crate::debugger::Breakpoint;
use crate::dwarf_data::DwarfData;
use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::size_of;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

impl Inferior {
    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        unsafe {
            ptrace::write(
                self.pid(),
                aligned_addr as ptrace::AddressType,
                updated_word as *mut std::ffi::c_void,
            )?;
        }
        Ok(orig_byte as u8)
    }

    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(
        target: &str,
        args: &Vec<String>,
        breakpoints: &mut HashMap<usize, Option<Breakpoint>>,
    ) -> Option<Inferior> {
        let mut cmd = Command::new(target);
        cmd.args(args);
        unsafe {
            cmd.pre_exec(child_traceme);
        }
        match cmd.spawn() {
            Ok(child) => {
                let mut inferior = Inferior { child };
                for (addr, breakpoint) in breakpoints {
                    // replacing the byte at breakpoint with the value 0xcc
                    // and record the original instrction's first byte
                    match inferior.write_byte(*addr, 0xcc) {
                        Ok(orig_byte) => {
                            *breakpoint = Some(Breakpoint {
                                addr: *addr,
                                orig_byte,
                            });
                        }
                        Err(_) => println!("Inferior::new can't write_byte {}", addr),
                    }
                }
                Some(inferior)
            }
            Err(_) => None,
        }
    }

    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid())?;
        let mut instruction_ptr: usize = regs.rip.try_into().unwrap();
        let mut base_ptr: usize = regs.rbp.try_into().unwrap();
        loop {
            let function = debug_data.get_function_from_addr(instruction_ptr).unwrap();
            let line = debug_data.get_line_from_addr(instruction_ptr).unwrap();
            println!("{} ({})", function, line);
            if function == "main" {
                break;
            }
            instruction_ptr =
                ptrace::read(self.pid(), (base_ptr + 8) as ptrace::AddressType)? as usize;
            base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as usize;
        }
        Ok(())
    }

    /// Wake up the inferior and wait for it to finish.
    pub fn continue_exec(
        &mut self,
        breakpoints: &HashMap<usize, Option<Breakpoint>>,
    ) -> Result<Status, nix::Error> {
        let mut regs = ptrace::getregs(self.pid())?;
        let rip: usize = regs.rip.try_into().unwrap(); // rip as usize
                                                       // check if inferior stopped at a breakpoint
        if let Some(breakpoint) = breakpoints.get(&(rip - 1)) {
            if let Some(bp) = breakpoint {
                let orig_byte = bp.orig_byte;
                println!("[inferior.continue_exec] Stopped at a breakpoint");
                // restore the first byte of the instruction we replaced
                self.write_byte(rip - 1, orig_byte).unwrap();
                // set %rip = %rip - 1 to rewind the instruction pointer
                regs.rip = (rip - 1) as u64;
                ptrace::setregs(self.pid(), regs).unwrap();
                // go to the next instruction
                ptrace::step(self.pid(), None).unwrap();
                // wait for inferior to stop due to SIGTRAP, just return if the inferior terminates here
                match self.wait(None).unwrap() {
                    Status::Exited(exit_code) => return Ok(Status::Exited(exit_code)),
                    Status::Signaled(signal) => return Ok(Status::Signaled(signal)),
                    Status::Stopped(_, _) => {
                        // restore 0xcc in the breakpoint location
                        self.write_byte(rip - 1, 0xcc).unwrap();
                    }
                }
            }
        }

        ptrace::cont(self.pid(), None)?; // Restart the stopped tracee process
        self.wait(None) // wait for inferior to stop or terminate
    }

    /// Kill the inferior(child process).
    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        println!("Killing running inferior (pid {})", self.pid());
        self.child.kill()
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }
}
