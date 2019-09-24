
use crate::evdev::{Device, UInputDevice};
use crate::foreign::*;
use super::{Error, Result};
use std::cell::Cell;

pub struct DestinationDevice {
    uidev: UInputDevice,
    state: DestinationDeviceState,
}

#[derive(Default)]
struct DestinationDeviceState {
    changed: Cell<bool>,
    rel_acc: Option<Vec<Cell<f32>>>,
}

// TODO: implement device capability
// pub struct DeviceCapability {
// }

impl DestinationDevice {
    pub fn new() -> Result<DestinationDevice> {
        let mut dev = Device::new().unwrap();

        dev.set_name("evdev device");

        {
            dev.enable_event(EV_REL, REL_X);
            dev.enable_event(EV_REL, REL_Y);
            dev.enable_event(EV_REL, REL_WHEEL);
            dev.enable_event(EV_REL, REL_HWHEEL);
            dev.enable_event(EV_REL, REL_WHEEL_HI_RES);
            dev.enable_event(EV_REL, REL_HWHEEL_HI_RES);

            dev.enable_event(EV_KEY, BTN_LEFT);
            dev.enable_event(EV_KEY, BTN_RIGHT);
            dev.enable_event(EV_KEY, BTN_MIDDLE);
            dev.enable_event(EV_KEY, BTN_SIDE);
            dev.enable_event(EV_KEY, BTN_EXTRA);
            dev.enable_event(EV_KEY, BTN_FORWARD);
            dev.enable_event(EV_KEY, BTN_BACK);
            dev.enable_event(EV_KEY, BTN_TASK);

            for code in 1..=248 {
                dev.enable_event(EV_KEY, code);
            }
        }

        let uidev = UInputDevice::new_from_device(dev)?;

        let mut state = DestinationDeviceState::default();
        state.rel_acc = Some(vec![Default::default(); REL_MAX as usize]);

        Ok(DestinationDevice { uidev, state })
    }

    pub fn write_event(&self, type_: u32, code: u32, value: i32) -> Result<()> {
        self.state.changed.set(true);
        Ok(self.uidev.write_event(type_, code, value)?)
    }

    pub fn move_relative(&self, code: u32, amount: f32) -> Result<()> {
        let cell = match &self.state.rel_acc {
            Some(map) =>
                match map.get(code as usize) {
                    Some(cell) => cell,
                    None =>
                        return Err(Error::Message(format!("invalid event code: {}", code).into())),
                },
            None =>
                return Err(Error::Message("invalid event type: EV_REL".into())),
        };

        let mut acc = cell.get() + amount;

        let trunc = acc.trunc();
        if trunc.abs() > 0.0f32 {
            self.write_event(EV_REL, code, trunc as i32)?;
            acc -= trunc;
        }

        cell.set(acc);
        
        Ok(())
    }

    pub fn sync(&self) {
        if self.state.changed.get() {
            let _ = self.write_event(EV_SYN, SYN_REPORT, 0);
            self.state.changed.set(false);
        }
    }
}
