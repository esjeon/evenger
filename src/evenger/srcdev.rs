
use crate::evdev::{Device, InputEvent, ReadFlag, ReadStatus};
use super::Result;
use std::os::unix::io::RawFd;
use std::path::Path;

pub struct SourceDevice {
    dev: Device,
}

impl SourceDevice {
    pub fn open<P: AsRef<Path>>(devpath: P) -> Result<SourceDevice> {
        use nix::fcntl::OFlag;
        use nix::sys::stat::Mode;
        let fd = nix::fcntl::open(
            devpath.as_ref(),
            OFlag::O_CLOEXEC | OFlag::O_NONBLOCK,
            Mode::empty()
        )?;

        let dev = Device::new_from_fd(fd)?;

        if let Err(e) = dev.grab(true) {
            eprintln!("cannot grab device {}: {}", devpath.as_ref().to_string_lossy(), e);
        }

        Ok(SourceDevice { dev })
    }

    pub fn fd(&self) -> RawFd {
        self.dev.fd()
            .expect("SourceDevice should be backed by an actual file")
    }

    pub fn read_event(&self) -> Result<Option<InputEvent>> {
        loop {
            match self.dev.next_event(ReadFlag::Normal)? {
                ReadStatus::Success(ev) => return Ok(Some(ev)),
                ReadStatus::Sync(_) => continue,
                ReadStatus::TryAgain => return Ok(None),
            }
        }
    }

    // TODO: specilaized functions: get_key_state, get_sw_state, etc
    pub fn get_event_state(&self, type_: u32, code: u32) -> Option<i32> {
        self.dev.fetch_event_value(type_, code)
    }
}
