
use nix::sys::epoll::*;
use std::os::unix::io::RawFd;
use std::time::Duration;

type Result<T> = std::result::Result<T, nix::Error>;

pub struct Muxer {
    epfd: RawFd,
}

pub struct MuxerEvents {
    buffer: [EpollEvent; 16],
    len: usize,
    index: usize,
}

pub struct MuxerEvent(EpollEvent);

impl Muxer {
    pub fn new() -> Result<Muxer> {
        Ok(Muxer {
            epfd: epoll_create()?,
        })
    }

    pub fn watch_input(&self, fd: RawFd) -> Result<()> {
        let mut epev = EpollEvent::new(EpollFlags::EPOLLIN, fd as u64);
        epoll_ctl(self.epfd, EpollOp::EpollCtlAdd, fd, &mut epev)?;
        Ok(())
    }

    pub fn wait(&self, timeout: Option<Duration>) -> Result<MuxerEvents> {
        let timeout_ms = match timeout {
            Some(dur) => dur.as_millis() as isize,
            None => -1,
        };

        let mut events = MuxerEvents::default();
        events.len = epoll_wait(self.epfd, &mut events.buffer, timeout_ms)?;

        Ok(events)
    }
}

impl Drop for Muxer {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.epfd);
    }
}

impl Default for MuxerEvents {
    fn default() -> Self {
        MuxerEvents {
            buffer: [EpollEvent::empty(); 16],
            len: 0,
            index: 0,
        }
    }
}

impl Iterator for MuxerEvents {
    type Item = MuxerEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None
        }

        let i = self.index;
        self.index += 1;
        Some(self.buffer[i].into())
    }
}

impl MuxerEvent {
    pub fn fd(&self) -> RawFd {
        self.0.data() as RawFd
    }

    pub fn readable(&self) -> bool {
        self.0.events().contains(EpollFlags::EPOLLIN)
    }

    pub fn hungup(&self) -> bool {
        self.0.events().contains(EpollFlags::EPOLLHUP)
    }
}

impl From<EpollEvent> for MuxerEvent {
    fn from(event: EpollEvent) -> Self {
        Self(event)
    }
}
