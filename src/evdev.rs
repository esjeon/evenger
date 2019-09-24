
use evdev_sys::*;
use nix::errno::Errno;
use std::ffi::CString;
use std::os::unix::io::RawFd;

type Result<T> = std::result::Result<T, nix::errno::Errno>;

pub struct Device(*mut libevdev);
pub struct UInputDevice(*mut libevdev_uinput);

pub struct InputEvent(input_event);

#[allow(unused)]
#[repr(i32)]
pub enum ReadFlag {
    Normal = LIBEVDEV_READ_FLAG_NORMAL,
    Sync = LIBEVDEV_READ_FLAG_SYNC,
}

pub enum ReadStatus {
    Success(InputEvent),
    Sync(InputEvent),
    TryAgain,
}

impl Device {
    pub fn new() -> Result<Self> {
        let raw = unsafe { libevdev_new() };
        if raw.is_null() {
            Err(Errno::ENOMEM)
        } else {
            Ok(Device(raw))
        }
    }

    pub fn new_from_fd(fd: RawFd) -> Result<Self> {
        let mut raw = 0 as *mut libevdev;
        match unsafe { libevdev_new_from_fd(fd, &mut raw) } {
            0 => Ok(Device(raw)),
            errno => Err(Errno::from_i32(-errno)),
        }
    }

    pub fn fd(&self) -> Option<RawFd> {
        match unsafe { libevdev_get_fd(self.0) } {
            -1 => None,
            fd => Some(fd),
        }
    }

    pub fn grab(&self, grab: bool) -> Result<()> {
        let grabmode: i32 = match grab {
            true => LIBEVDEV_GRAB,
            false => LIBEVDEV_UNGRAB,
        };

        match unsafe { libevdev_grab(self.0, grabmode) } {
            0 => Ok(()),
            errno => Err(Errno::from_i32(-errno)),
        }
    }

    pub fn next_event(&self, flag: ReadFlag) -> Result<ReadStatus> {
        const NEG_EAGAIN: i32 = -(nix::errno::Errno::EAGAIN as i32);

        let mut event = unsafe { InputEvent::uninitialized() };
        match unsafe { libevdev_next_event(self.0, flag as u32, event.raw_mut()) } {
            LIBEVDEV_READ_STATUS_SUCCESS =>
                Ok(ReadStatus::Success(event)),
            LIBEVDEV_READ_STATUS_SYNC =>
                Ok(ReadStatus::Sync(event)),
            NEG_EAGAIN =>
                Ok(ReadStatus::TryAgain),
            neg_errno =>
                Err(nix::errno::Errno::from_i32(-neg_errno)),
        }
    }

    pub fn fetch_event_value(&self, type_: u32, code: u32) -> Option<i32> {
        let mut value = 0;
        match unsafe {
            libevdev_fetch_event_value(self.0, type_, code, &mut value)
        } {
            1 => Some(value),
            0 => None,
            _ => None,
        }
    }

    pub fn enable_event(&mut self, type_: u32, code: u32) {
        match unsafe {
            libevdev_enable_event_code(self.0, type_, code, 0 as *const _)
        } {
            0 => {},
            -1 => println!("warning: cannot enable input code {:?}", code),
            _ => {},
        }
    }

    pub fn set_name<T: Into<Vec<u8>>>(&mut self, name: T) {
        let cstr = CString::new(name).unwrap();
        unsafe { libevdev_set_name(self.0, cstr.as_ptr()) };
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let opt_fd = self.fd();
        unsafe { libevdev_free(self.0) };

        if let Some(fd) = opt_fd {
            let _ = nix::unistd::close(fd);
        }
    }
}

impl UInputDevice {
    pub fn new_from_device(dev: Device) -> Result<Self> {
        let mut raw = 0 as *mut libevdev_uinput;
        match unsafe {
            libevdev_uinput_create_from_device(dev.0, LIBEVDEV_UINPUT_OPEN_MANAGED, &mut raw)
        } {
            0 => Ok(Self(raw)),
            neg_errno => Err(nix::errno::Errno::from_i32(-neg_errno)),
        }
    }

    pub fn write_event(&self, type_: u32, code: u32, value: i32) -> Result<()> {
        match unsafe {
            libevdev_uinput_write_event(self.0, type_, code, value)
        } {
            0 => Ok(()),
            neg_errno => Err(nix::errno::Errno::from_i32(-neg_errno)),
        }
    }
}

impl Drop for UInputDevice {
    fn drop(&mut self) {
        unsafe { libevdev_uinput_destroy(self.0) };
    }
}

impl InputEvent {
    // fn new() -> Self {
    //     InputEvent(
    //         input_event {
    //             time: timeval {
    //                 tv_sec: 0,
    //                 tv_usec: 0,
    //             },
    //             type_: 0,
    //             code: 0,
    //             value: 0,
    //         }
    //     )
    // }

    unsafe fn uninitialized() -> Self {
        InputEvent(std::mem::uninitialized())
    }

    pub fn raw_mut(&mut self) -> &mut input_event {
        &mut self.0
    }

    pub fn type_(&self) -> u32 {
        self.0.type_ as u32
    }

    pub fn code(&self) -> u32 {
        self.0.code as u32
    }

    pub fn value(&self) -> i32 {
        self.0.value
    }
}
