
use crate::evdev::{InputEvent};
use crate::muxer;
use super::{Error, Result};
use super::destdev::{DestinationDevice};
use super::srcdev::{SourceDevice, Modifier};
use muxer::Muxer;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::path::Path;

pub struct Evenger {
    muxer: Muxer,
    srcdevs: HashMap<RawFd, SourceDevice>,
    destdev: DestinationDevice,
}

impl Evenger {
    pub fn new() -> Result<Evenger> {
        // TODO: configurable output device

        Ok(Evenger {
            muxer: Muxer::new()
                .map_err(|e| Error::Description("muxer".into(), Box::new(e)))?,
            srcdevs: HashMap::new(),
            destdev: DestinationDevice::new()
                .map_err(|e| Error::Description("destdev".into(), Box::new(e)))?,
        })
    }

    pub fn open_device<P: AsRef<Path>>(&mut self, devpath: P) -> Result<()> {
        let srcdev = SourceDevice::open(devpath)?;
        let fd = srcdev.fd();

        self.muxer.watch_input(fd)?;
        self.srcdevs.insert(fd, srcdev);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            for mux_ev in self.muxer.wait(None)? {
                if mux_ev.readable() {
                    self.on_srcdev_ready(mux_ev.fd())?;
                }

                if mux_ev.hungup() {
                    self.srcdevs.remove(&mux_ev.fd());
                }
            }

            if self.srcdevs.len() == 0 {
                break
            }
        }

        Ok(())
    }

    fn on_srcdev_ready(&self, fd: RawFd) -> Result<()> {
        let srcdev = self.srcdevs.get(&fd)
            .ok_or_else(|| Error::Message("invalid fd".into()))?;

        loop {
            match srcdev.read_event()? {
                Some(event) => {
                    self.translate_event(srcdev, &event)?;
                },
                None => break,
            }
        }

        Ok(())
    }

    fn translate_event(&self, srcdev: &SourceDevice, event: &InputEvent) -> Result<()> {
        use crate::foreign::*;

        let evtype = event.type_();
        let evcode = event.code();
        match (evtype, evcode) {
            (EV_REL, REL_Y) => {
                if Some(true) == srcdev.match_modifier(Modifier::Key(BTN_TASK)) {
                    /* mapping REL to REL */
                    self.destdev.move_relative(REL_WHEEL,
                        event.value() as f32 / -16.0f32)?;
                    return Ok(());
                }
            },
            (EV_KEY, KEY_CAPSLOCK) => {
                if Some(true) == srcdev.match_modifier(Modifier::Led(LED_CAPSL)) {
                    if event.value() == /* down */1  {
                        /* ignore */
                        return Ok(());
                    }
                }
            },
            (EV_KEY, KEY_LEFTSHIFT) => {
                if Some(true) == srcdev.match_modifier(Modifier::Led(LED_CAPSL)) {
                    self.destdev.press_key(KEY_CAPSLOCK, true)?;
                    self.destdev.press_key(KEY_CAPSLOCK, false)?;
                }
            },
            _ => {},
        };

        if let Err(e) = self.destdev.write_event(evtype, evcode, event.value()) {
            eprintln!("passthru failure (type={} code={} value={}): {}",
                evtype, evcode, event.value(), e);
        }

        Ok(())
    }
}
