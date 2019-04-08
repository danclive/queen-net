use std::os::raw::c_char;
use std::ffi::c_void;

pub const AF_INET: i32 = 2;
pub const AF_INET6: i32 = 10;
pub const SOCK_STREAM: i32 = 1;
pub const IPPRPTO_TCP: i32 = 6;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [c_char; 14],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: in_addr,
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct in_addr {
    pub s_addr: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sockaddr_in6 {
    pub sin6_family: u16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct in6_addr {
    pub s6_addr: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct sockaddr_storage {
    pub ss_family: u16,
    _unused: [u8; 126]
}

extern {
    pub fn socket(fanily: i32, ty: i32, protocol: i32) -> i32;
    pub fn connect(sockfd: i32, servaddr: *const sockaddr, addrlen: u32) -> i32;
    pub fn bind(sockfd: i32, myaddr: *const sockaddr, addrlen: u32) -> i32;
    pub fn listen(sockfd: i32, backlog: i32);
    pub fn accept(sockfd: i32, cliaddr: *mut sockaddr, addrlen: *mut u32) -> i32;
    pub fn close(sockfd: i32) -> i32;
    pub fn getsockname(sockfd: i32, localaddr: *mut sockaddr, addrlen: *mut u32) -> i32;
    pub fn getpeername(sockfd: i32, peeraddr: *mut sockaddr, addrlen: *mut u32) -> i32;
    pub fn read(fd: i32, buf: *mut c_void, count: isize) -> i32;
    pub fn write(fd: i32, buf: *const c_void, count: isize) -> i32;
}

fn main() {
    use std::io::Error;
    use std::mem;
    use std::thread;
    use std::time::Duration;

    thread::spawn(|| {

        // server
        unsafe {
            let socket = socket(AF_INET, SOCK_STREAM, IPPRPTO_TCP);
            if socket < 0 {
                panic!("last OS error: {:?}", Error::last_os_error());
            }

            let servaddr = sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: 8080u16.to_be(),
                sin_addr: in_addr {
                    s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be()
                },
                sin_zero: mem::zeroed()
            };

            let result = bind(socket, &servaddr as *const sockaddr_in as *const sockaddr, mem::size_of_val(&servaddr) as u32);
            if result < 0 {
                println!("last OS error: {:?}", Error::last_os_error());
                close(socket);
            }

            listen(socket, 128);

            loop {
                let mut cliaddr: sockaddr_storage = mem::zeroed();
                let mut len = mem::size_of_val(&cliaddr) as u32;

                let client_socket = accept(socket, &mut cliaddr as *mut sockaddr_storage as *mut sockaddr, &mut len);
                if client_socket < 0 {
                    println!("last OS error: {:?}", Error::last_os_error());
                    break;
                }

                thread::spawn(move || {
                    loop {
                        let mut buf = [0u8; 64];
                        let n = read(client_socket, &mut buf as *mut _ as *mut c_void, buf.len() as isize);
                        if n <= 0 {
                            break;
                        }

                        println!("{:?}", String::from_utf8_lossy(&buf[0..n as usize]));

                        let msg = b"Hi, client!";
                        let n = write(client_socket, msg as *const _ as *const c_void, msg.len() as isize);
                        if n <= 0 {
                            break;
                        }
                    }

                    close(client_socket);
                });
            }

            close(socket);
        }

    });

    thread::sleep(Duration::from_secs(1));

    // client
    unsafe {
        let socket = socket(AF_INET, SOCK_STREAM, IPPRPTO_TCP);
        if socket < 0 {
            panic!("last OS error: {:?}", Error::last_os_error());
        }

        let servaddr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 8080u16.to_be(),
            sin_addr: in_addr {
                s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be()
            },
            sin_zero: mem::zeroed()
        };

        let result = connect(socket, &servaddr as *const sockaddr_in as *const sockaddr, mem::size_of_val(&servaddr) as u32);
        if result < 0 {
            println!("last OS error: {:?}", Error::last_os_error());
            close(socket);
        }

        let msg = b"Hello, server!";
        let n = write(socket, msg as *const _ as *const c_void, msg.len() as isize);
        if n <= 0 {
            println!("last OS error: {:?}", Error::last_os_error());
            close(socket);
        }

        let mut buf = [0u8; 64];
        let n = read(socket, &mut buf as *mut _ as *mut c_void, buf.len() as isize);
        if n <= 0 {
            println!("last OS error: {:?}", Error::last_os_error());
        }

        println!("{:?}", String::from_utf8_lossy(&buf[0..n as usize]));

        close(socket);
    }
}
