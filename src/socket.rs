use std::io;
use std::mem;
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};

use libc as c;

use crate::fd::FileDesc;
use crate::util::cvt;

pub struct Socket(FileDesc);

impl Socket {
    pub fn new(family: c::c_int, ty: c::c_int, protocol: c::c_int) -> io::Result<Socket> {
        unsafe {
            cvt(c::socket(family, ty | c::SOCK_CLOEXEC, protocol))
                .map(|fd| Socket(FileDesc::from_raw_fd(fd)))
        }
    }

    pub fn bind(&self, storage: *const c::sockaddr, len: c::socklen_t) -> io::Result<()> {
        self.setsockopt(c::SOL_SOCKET, c::SO_REUSEADDR, 1)?;

        cvt(unsafe { c::bind(self.0.raw(), storage, len) })?;

        Ok(())
    }

    pub fn listen(&self, backlog: c::c_int) -> io::Result<()> {
        cvt(unsafe { c::listen(self.0.raw(), backlog) })?;
        Ok(())
    }

    pub fn accept(&self, storage: *mut c::sockaddr, len: *mut c::socklen_t) -> io::Result<Socket> {
        let fd = cvt(unsafe { c::accept4(self.0.raw(), storage, len, c::SOCK_CLOEXEC) })?;
        Ok(Socket(unsafe { FileDesc::from_raw_fd(fd) }))
    }

    pub fn connect(&self, storage: *const c::sockaddr, len: c::socklen_t) -> io::Result<()> {
        cvt(unsafe { c::connect(self.0.raw(), storage, len) })?;
        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    pub fn get_cloexec(&self) -> io::Result<bool> {
        self.0.get_cloexec()
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        self.0.set_cloexec()
    }

    pub fn setsockopt<T>(&self, opt: c::c_int, val: c::c_int, payload: T) -> io::Result<()> {
        unsafe {
            let payload = &payload as *const T as *const c::c_void;

            cvt(c::setsockopt(
                self.0.raw(),
                opt,
                val,
                payload,
                mem::size_of::<T>() as c::socklen_t
            ))?;

            Ok(())
        }
    }

    pub fn getsockopt<T: Copy>(&self, opt: c::c_int, val: c::c_int) -> io::Result<T> {
        unsafe {
            let mut slot: T = mem::zeroed();
            let mut len = mem::size_of::<T>() as c::socklen_t;

            cvt(c::getsockopt(
                self.0.raw(),
                opt,
                val,
                &mut slot as *mut T as *mut c::c_void,
                &mut len
            ))?;

            assert_eq!(len as usize, mem::size_of::<T>());
            Ok(slot)
        }
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Socket {
        Socket(FileDesc::from_raw_fd(fd))
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.raw()
    }
}
