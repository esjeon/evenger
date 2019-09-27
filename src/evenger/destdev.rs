
use crate::evdev::{Device, UInputDevice};
use crate::foreign::*;
use super::{DeviceId, Error, Result};
use std::cell::Cell;

pub struct DestinationDevice {
    id: DeviceId,
    uidev: UInputDevice,
    components: InternalComponents,
    should_sync: Cell<bool>,
}

#[derive(Default)]
struct InternalComponents {
    relative: Option<Vec<Cell<RelativeComponent>>>,
    key: Option<Vec<Cell<KeyComponent>>>,
}

#[derive(Clone, Copy, Default)]
struct RelativeComponent {
    acc: f32,
}

#[derive(Clone, Copy, Default)]
struct KeyComponent {
    pressed: bool,
}

// TODO: implement device capability
// pub struct DeviceCapability {
// }

impl DestinationDevice {
    pub fn new(id: DeviceId) -> Result<DestinationDevice> {
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

        let mut components = InternalComponents::default();
        components.relative = Some(vec![Default::default(); REL_CNT as usize]);
        components.key = Some(vec![Default::default(); KEY_CNT as usize]);

        Ok(DestinationDevice {
            id,
            uidev,
            components, 
            should_sync: Cell::from(false),
        })
    }

    pub fn id(&self) -> DeviceId {
        self.id.clone()
    }

    pub fn write_event(&self, type_: u32, code: u32, value: i32) -> Result<()> {
        self.should_sync.set(true);
        Ok(self.uidev.write_event(type_, code, value)?)
    }

    pub fn move_relative(&self, code: u32, amount: f32) -> Result<()> {
        let component_cell =
            self.components.relative.as_ref()
                .ok_or_else(|| Error::Message("invalid component: Relative".into()))?
            .get(code as usize)
                .ok_or_else(|| Error::Message(format!("invalid event code: {}", code)))?
            ;

        let mut component = component_cell.get();
        let mut acc = component.acc + amount;
        let trunc = acc.trunc();
        if trunc.abs() > 0.0f32 {
            self.write_event(EV_REL, code, trunc as i32)?;
            acc -= trunc;
        }
        component.acc = acc;
        component_cell.set(component);
        
        Ok(())
    }

    pub fn press_key(&self, code: u32, press: bool) -> Result<()> {
        let component_cell =
            self.components.key.as_ref()
                .ok_or_else(|| Error::Message("invalid component: Relative".into()))?
            .get(code as usize)
                .ok_or_else(|| Error::Message(format!("invalid event code: {}", code)))?
            ;
        
        let mut component = component_cell.get();
        if component.pressed != press {
            self.write_event(EV_KEY, code, if press { 1 } else { 0 })?;

            component.pressed = press;
            component_cell.set(component);
        }

        Ok(())
    }

    pub fn sync(&self) {
        if self.should_sync.get() {
            let _ = self.write_event(EV_SYN, SYN_REPORT, 0);
            self.should_sync.set(false);
        }
    }
}
