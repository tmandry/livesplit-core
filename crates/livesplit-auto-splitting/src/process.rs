use std::{
    io,
    time::{Duration, Instant},
};

use proc_maps::{MapRange, Pid};
use read_process_memory::{CopyAddress, ProcessHandle};
use snafu::{OptionExt, ResultExt, Snafu};
use sysinfo::{self, ProcessExt};

use crate::runtime::ProcessList;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)))]
pub enum OpenError {
    ProcessDoesntExist,
    InvalidHandle { source: io::Error },
}

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)))]
pub enum ModuleError {
    ModuleDoesntExist,
    ListModules { source: io::Error },
}

pub type Address = u64;

pub struct Process {
    handle: ProcessHandle,
    pid: Pid,
    modules: Vec<MapRange>,
    last_check: Instant,
}

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process").field("pid", &self.pid).finish()
    }
}

impl Process {
    pub fn with_name(name: &str, process_list: &mut ProcessList) -> Result<Self, OpenError> {
        process_list.refresh();
        let processes = process_list.process_by_name(name);

        let pid = processes.first().context(ProcessDoesntExist)?.pid() as Pid;

        let handle = pid.try_into().context(InvalidHandle)?;

        Ok(Process {
            handle,
            pid,
            modules: Vec::new(),
            last_check: Instant::now() - Duration::from_secs(1),
        })
    }

    pub fn is_open(&self, process_list: &mut ProcessList) -> bool {
        // FIXME: We can actually ask the list to only refresh the individual process.
        process_list.refresh();
        process_list.is_open(self.pid as _)
    }

    pub fn module_address(&mut self, module: &str) -> Result<Address, ModuleError> {
        let now = Instant::now();
        if now - self.last_check >= Duration::from_secs(1) {
            self.modules = match proc_maps::get_process_maps(self.pid) {
                Ok(m) => m,
                Err(source) => {
                    self.modules.clear();
                    return Err(ModuleError::ListModules { source });
                }
            };
            self.last_check = now;
        }
        self.modules
            .iter()
            .find(|m| m.filename().map_or(false, |f| f.ends_with(module)))
            .context(ModuleDoesntExist)
            .map(|m| m.start() as u64)
    }

    pub fn read_mem(&self, address: Address, buf: &mut [u8]) -> io::Result<()> {
        self.handle.copy_address(address as usize, buf)
    }
}