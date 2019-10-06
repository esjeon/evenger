
use crate::muxer;
use super::{Error, Result};
use super::destdev::{DestinationDevice};
use super::srcdev::{SourceDeviceSet, SourceDevice, Event, Modifier};
use muxer::Muxer;
use std::{path::Path, rc::Rc};
use std::os::unix::io::RawFd;

pub struct Evenger {
    muxer: Muxer,
    srcdevs: SourceDeviceSet,
    destdev: DestinationDevice,
}

impl Evenger {
    pub fn new() -> Result<Evenger> {
        // TODO: configurable output device

        let muxer = Muxer::new()
            .map_err(|e| Error::Description("muxer".into(), Box::new(e)))?;

        let destdev = DestinationDevice::new(Rc::new("output".to_string()))
            .map_err(|e| Error::Description("destdev".into(), Box::new(e)))?;

        Ok(Evenger {
            muxer,
            srcdevs: SourceDeviceSet::new(),
            destdev,
        })
    }

    pub fn open_device<S, P>(&mut self, id: S, devpath: P) -> Result<()> 
        where S: Into<String>,
              P: AsRef<Path>,
    {
        let id = Rc::new(id.into());

        let srcdev = SourceDevice::open(Rc::clone(&id), devpath)?;
        let fd = srcdev.fd();

        self.muxer.watch_input(fd)?;
        self.srcdevs.push(srcdev);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            for mux_ev in self.muxer.wait(None)? {
                if mux_ev.readable() {
                    self.on_srcdev_ready(mux_ev.fd())?;
                }

                if mux_ev.hungup() {
                    self.srcdevs.remove_by_fd(mux_ev.fd());
                }
            }

            if self.srcdevs.len() == 0 {
                break
            }
        }

        Ok(())
    }

    fn on_srcdev_ready(&self, fd: RawFd) -> Result<()> {
        let srcdev = self.srcdevs.get_by_fd(fd)
            .ok_or_else(|| Error::msg("invalid fd"))?;

        loop {
            match srcdev.read_event()? {
                Some(event) => {
                    self.translate_event(&event)?;
                },
                None => break,
            }
        }

        Ok(())
    }

    fn translate_event(&self, event: &Event) -> Result<()> {
        use crate::foreign::*;

        let target = event.target();

        match (target.type_(), target.code()) {
            (EV_REL, REL_Y) => {
                let mouse_dev = self.srcdevs.get_by_id(Rc::new("mouse".to_string()))
                    .ok_or_else(|| Error::msg("can't get device 'mouse'"))?;

                if Some(true) == mouse_dev.match_modifier(Modifier::Key(BTN_TASK, true)) {
                    /* mapping REL to REL */
                    self.destdev.move_relative(REL_WHEEL,
                        event.value() as f32 / -16.0f32)?;
                    return Ok(());
                }
            },
            (EV_KEY, KEY_CAPSLOCK) => {
                let keyboard_dev = self.srcdevs.get_by_id(Rc::new("keyboard".to_string()))
                    .ok_or_else(|| Error::msg("can't get device 'keyboard'"))?;

                if Some(true) == keyboard_dev.match_modifier(Modifier::Led(LED_CAPSL, true)) {
                    if event.value() == /* down */1  {
                        /* ignore */
                        return Ok(());
                    }
                }
            },
            (EV_KEY, KEY_LEFTSHIFT) => {
                let keyboard_dev = self.srcdevs.get_by_id(Rc::new("keyboard".to_string()))
                    .ok_or_else(|| Error::msg("can't get device 'keyboard'"))?;

                if Some(true) == keyboard_dev.match_modifier(Modifier::Led(LED_CAPSL, true)) {
                    self.destdev.press_key(KEY_CAPSLOCK, true)?;
                    self.destdev.press_key(KEY_CAPSLOCK, false)?;
                }
            },
            _ => {},
        };

        if let Err(e) = self.destdev.write_event(target.type_(), target.code(), event.value()) {
            eprintln!("passthru failure (type={} code={} value={}): {}",
                target.type_(), target.code(), event.value(), e);
        }

        Ok(())
    }
}
