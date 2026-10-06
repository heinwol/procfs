//! Refer to the [kernel docs](https://docs.kernel.org/filesystems/proc.html#proc-consoles)
//! for more info

use std::io::BufRead;

use bitflags::bitflags;
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

use crate::ProcResult;

bitflags! {
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct ConsoleOperations: u32 {
        /// Can do read operations
        const READ    = 1;
        /// Can do write operations
        const WRITE   = 1 << 1;
        /// Can do unblank
        const UNBLANK = 1 << 2;
    }
}

impl ConsoleOperations {
    fn from_str(s: &str) -> ProcResult<Self> {
        let mut console_operations = ConsoleOperations::empty();
        for ch in s.chars() {
            match ch {
                'R' => console_operations.insert(ConsoleOperations::READ),
                'W' => console_operations.insert(ConsoleOperations::WRITE),
                'U' => console_operations.insert(ConsoleOperations::UNBLANK),
                c @ _ if c.is_whitespace() || c == '-' => continue,
                c @ _ => return Err(build_internal_error!(format!("Unknown console operation: {c}"))),
            }
        }
        Ok(console_operations)
    }
}

bitflags! {
    /// Flags for device properties.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct ConsoleFlags: u32 {
        /// It is enabled: E
        const ENABLED   = 1;
        /// It is preferred console: C
        const PREFERRED = 1 << 1;
        /// It is primary boot console: B
        const BOOT      = 1 << 2;
        /// It is used for printk buffer: p
        const PRINTK    = 1 << 3;
        /// It is not a TTY but a Braille device: b
        const BRAILE    = 1 << 4;
        /// It is safe to use when cpu is offline: a
        const SAFE      = 1 << 5;
    }
}

impl ConsoleFlags {
    fn from_str(s: &str) -> ProcResult<Self> {
        let mut console_operations = ConsoleFlags::empty();
        for ch in s.chars() {
            match ch {
                'E' => console_operations.insert(ConsoleFlags::ENABLED),
                'C' => console_operations.insert(ConsoleFlags::PREFERRED),
                'B' => console_operations.insert(ConsoleFlags::BOOT),
                'p' => console_operations.insert(ConsoleFlags::PRINTK),
                'b' => console_operations.insert(ConsoleFlags::BRAILE),
                'a' => console_operations.insert(ConsoleFlags::SAFE),
                c @ _ if c.is_whitespace() => continue,
                c @ _ => return Err(build_internal_error!(format!("Unknown flag: {c}"))),
            }
        }
        Ok(console_operations)
    }
}

/// Major and minor numbers of the console character device
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct ConsoleDeviceNumber {
    pub major: u32,
    pub minor: u32,
}

impl ConsoleDeviceNumber {
    fn from_str(s: &str) -> ProcResult<Self> {
        let mut split = s.trim().split(":");
        let major = from_str!(u32, expect!(split.next()));
        let minor = from_str!(u32, expect!(split.next()));
        Ok(ConsoleDeviceNumber { major, minor })
    }
}

/// Console character device representation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Console {
    /// Console character device name (like `tty0` or `ttyS0`)
    pub name: String,
    pub operations: ConsoleOperations,
    pub flags: ConsoleFlags,
    pub device: ConsoleDeviceNumber,
}

impl Console {
    /// This parses one line of `/proc/consoles` with a result
    /// of a single console device
    fn from_line(line: &str) -> ProcResult<Self> {
        // split str into 3 parts
        let mut split = line.split(|c| c == '(' || c == ')');

        let name_and_operations = expect!(split.next());
        let (name, operations) = {
            let mut split = name_and_operations.split_whitespace();
            let name = expect!(split.next());
            let operations_str = expect!(split.next());
            (name.to_owned(), expect!(ConsoleOperations::from_str(operations_str)))
        };

        let flags = expect!(ConsoleFlags::from_str(expect!(split.next())));
        let device = expect!(ConsoleDeviceNumber::from_str(expect!(split.next())));
        Ok(Console {
            name,
            operations,
            flags,
            device,
        })
    }
}

/// A list of console character devices.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Consoles(pub Vec<Console>);

impl crate::FromBufRead for Consoles {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut v = Vec::new();

        for line in r.lines() {
            let line = line?;
            if !line.is_empty() {
                v.push(Console::from_line(&line)?);
            }
        }
        Ok(Consoles(v))
    }
}
