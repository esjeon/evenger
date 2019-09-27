
use crate::evdev::{Device, InputEvent, ReadFlag, ReadStatus};
use crate::foreign::*;
use super::{DeviceId, Result};
use std::{path::Path, rc::Rc};
use std::os::unix::io::RawFd;

pub struct SourceDevice {
    id: DeviceId,
    dev: Device,
}

pub struct Event {
    srcdev_id: DeviceId,
    base: InputEvent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventTarget(u32, u32);

pub enum Modifier {
    Key(u32),
    // TODO: Abs(u32), // min/max/resoultin? multitouch?
    Led(u32),
    Switch(u32),
}

impl SourceDevice {
    pub fn open<P: AsRef<Path>>(id: DeviceId, devpath: P) -> Result<SourceDevice> {
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

        Ok(SourceDevice { id, dev })
    }

    pub fn id(&self) -> DeviceId {
        Rc::clone(&self.id)
    }

    pub fn fd(&self) -> RawFd {
        self.dev.fd()
            .expect("SourceDevice should be backed by an actual file")
    }

    pub fn read_event(&self) -> Result<Option<Event>> {
        loop {
            match self.dev.next_event(ReadFlag::Normal)? {
                ReadStatus::Success(ev) => {
                    let event = Event::new(self.id(), ev);
                    return Ok(Some(event))
                },
                ReadStatus::Sync(_) => continue,
                ReadStatus::TryAgain => return Ok(None),
            }
        }
    }

    // TODO: specilaized functions: get_key_state, get_sw_state, etc
    pub fn get_event_state(&self, type_: u32, code: u32) -> Option<i32> {
        self.dev.fetch_event_value(type_, code)
    }

    pub fn match_modifier(&self, modf: Modifier) -> Option<bool> {
        let (type_, code) = match modf {
            Modifier::Key(code)    => (EV_KEY, code),
            Modifier::Led(code)    => (EV_LED, code),
            Modifier::Switch(code) => (EV_SW, code),
            // _ => return None,
        };

        match self.get_event_state(type_, code) {
            Some(v) => Some(v != 0),
            None => None,
        }
    }
}

impl Event {
    pub fn new(srcdev_id: DeviceId, base: InputEvent) -> Self {
        Self { srcdev_id, base }
    }

    pub fn srcdev_id(&self) -> DeviceId {
        Rc::clone(&self.srcdev_id)
    }

    pub fn target(&self) -> EventTarget {
        EventTarget::new(self.base.type_(), self.base.code())
    }

    pub fn value(&self) -> i32 {
        self.base.value()
    }
}

impl EventTarget {
    pub fn new(type_: u32, code: u32) -> Self {
        Self(type_, code)
    }
    pub fn type_(&self) -> u32 { self.0 }
    pub fn code(&self) -> u32 { self.1 }
}
