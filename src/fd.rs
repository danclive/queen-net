use std::mem;
use std::io;
use std::cmp;
use std::os::unix::io::FromRawFd;

use libc as c;

use crate::util::cvt;

#[derive(Debug)]
pub struct FileDesc(c::c_int);

pub fn max_len() -> usize {
    <c::ssize_t>::max_value() as usize
}

impl FileDesc {
    pub fn raw(&self) -> c::c_int {
        self.0
    }

    pub fn into_raw(self) -> c::c_int {
        let fd = self.0;
        mem::forget(self);
        fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            c::read(
                self.0,
                buf.as_mut_ptr() as *mut c::c_void,
                cmp::min(buf.len(), max_len())
            )
        })?;

        Ok(ret as usize)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            c::write(
                self.0,
                buf.as_ptr() as *const c::c_void,
                cmp::min(buf.len(), max_len())
            )
        })?;

        Ok(ret as usize)
    }

    pub fn get_cloexec(&self) -> io::Result<bool> {
        unsafe {
            Ok((cvt(libc::fcntl(self.0, c::F_GETFD))? & libc::FD_CLOEXEC) != 0)
        }
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            cvt(c::ioctl(self.0, c::FIOCLEX))?;
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let v = nonblocking as c::c_int;
            cvt(c::ioctl(self.0, c::FIONBIO, &v))?;
            Ok(())
        }
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        cvt(unsafe { c::fcntl(self.0, c::F_DUPFD_CLOEXEC, 0) }).and_then(|fd| {
            let fd = FileDesc(fd);
            Ok(fd)
        })
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(fd: c::c_int) -> FileDesc {
        FileDesc(fd)
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        let _ = unsafe { c::close(self.0) };
    }
}
