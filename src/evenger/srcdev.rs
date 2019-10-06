
use crate::evdev::{Device, InputEvent, ReadFlag, ReadStatus};
use crate::foreign::*;
use super::{DeviceId, Result};
use std::{path::Path, rc::Rc, rc::Weak};
use std::collections::HashMap;
use std::os::unix::io::RawFd;

#[derive(Default)]
pub struct SourceDeviceSet {
    fdmap: HashMap<RawFd, Rc<SourceDevice>>,
    idmap: HashMap<DeviceId, Weak<SourceDevice>>,
}

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

#[derive(Clone, PartialEq)]
pub enum Modifier {
    Key(u32, bool),
    // TODO: Abs(u32), // min/max/resoultin? multitouch?
    Led(u32, bool),
    Switch(u32, bool),
}


impl SourceDeviceSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.fdmap.len()
    }

    pub fn push(&mut self, srcdev: SourceDevice) {
        let (id, fd) = (srcdev.id(), srcdev.fd());
        let srcdev = Rc::new(srcdev);

        self.fdmap.insert(fd, Rc::clone(&srcdev));
        self.idmap.insert(id, Rc::downgrade(&srcdev));
    }

    pub fn remove_by_fd(&mut self, fd: RawFd) {
        if let Some(srcdev) = self.get_by_fd(fd) {
            self.fdmap.remove(&fd);
            self.idmap.remove(&srcdev.id());
        }
    }

    pub fn get_by_id(&self, id: DeviceId) -> Option<Rc<SourceDevice>> {
        match self.idmap.get(&id) {
            Some(weak) => weak.upgrade(),
            None => None,
        }
    }

    pub fn get_by_fd(&self, fd: RawFd) -> Option<Rc<SourceDevice>> {
        self.fdmap.get(&fd).map(|rc| rc.clone())
    }

    pub fn test_modifier(&self, device: Option<DeviceId>, modf: Modifier) -> bool {
        match device {
            Some(id) => self.get_by_id(id)
                            .map(|srcdev|
                                srcdev.match_modifier(modf)
                                      .unwrap_or(false))
                            .unwrap_or(false),
            None => self.fdmap.values()
                        .any(|srcdev|
                            srcdev.match_modifier(modf.clone())
                                  .unwrap_or(false)),
        }  
    }
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
        let (type_, code, val) = match modf {
            Modifier::Key(code, true)     => (EV_KEY, code, 1),
            Modifier::Key(code, false)    => (EV_KEY, code, 0),
            Modifier::Led(code, true)     => (EV_LED, code, 1),
            Modifier::Led(code, false)    => (EV_LED, code, 0),
            Modifier::Switch(code, true)  => (EV_SW , code, 1),
            Modifier::Switch(code, false) => (EV_SW , code, 0),
            _ => return None,
        };

        match self.get_event_state(type_, code) {
            Some(v) => Some(v == val),
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
