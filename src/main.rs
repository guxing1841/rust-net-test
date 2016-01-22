extern crate libc as c;
use std::mem as mem;
trait NetInt {
    fn from_be(i: Self) -> Self;
    fn to_be(&self) -> Self;
}

pub trait IntoInner<Inner> {
    fn into_inner(self) -> Inner;
}

macro_rules! doit {
    ($($t:ident)*) => ($(impl NetInt for $t {
        fn from_be(i: Self) -> Self { <$t>::from_be(i) }
        fn to_be(&self) -> Self { <$t>::to_be(*self) }
    })*)
}
// doit 宏绑定
doit! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }

fn hton<I: NetInt>(i: I) -> I { i.to_be() }
fn ntoh<I: NetInt>(i: I) -> I { I::from_be(i) }

fn test_connect1() {
    let fd;
    unsafe {
        fd = c::socket(c::AF_INET, c::SOCK_STREAM, c::IPPROTO_TCP);
    }
    let servaddr = c::sockaddr_in {
	sin_family: c::AF_INET as c::sa_family_t,
	sin_port: hton(80),
	sin_addr: c::in_addr {
            s_addr: hton(((127 as u32) << 24) |
                         ((0 as u32) << 16) |
                         ((0 as u32) << 8) |
                         (1 as u32))
        },
        .. unsafe { mem::zeroed() }
    };
    unsafe {
        c::connect(fd, (&servaddr as *const _ as *const _), mem::size_of_val(&servaddr) as c::socklen_t);
    }    
}

// ？Sized 可同时支持sized和unsized类型
/*
fn foo<T>() {} // can only be used with sized T
fn bar<T: ?Sized>() {} // can be used with both sized and unsized T
 */
pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

pub struct Ipv4Addr {
    inner: c::in_addr,
}

pub struct Ipv6Addr {
    inner: c::in6_addr,
}

impl Ipv4Addr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        Ipv4Addr {
            inner: c::in_addr {
                s_addr: hton(((a as u32) << 24) |
                             ((b as u32) << 16) |
                             ((c as u32) <<  8) |
                              (d as u32)),
            }
        }
    }
}

impl Ipv6Addr {
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16,
               h: u16) -> Ipv6Addr {
        let mut addr: c::in6_addr = unsafe { mem::zeroed() };
        addr.s6_addr = [(a >> 8) as u8, a as u8,
                        (b >> 8) as u8, b as u8,
                        (c >> 8) as u8, c as u8,
                        (d >> 8) as u8, d as u8,
                        (e >> 8) as u8, e as u8,
                        (f >> 8) as u8, f as u8,
                        (g >> 8) as u8, g as u8,
                        (h >> 8) as u8, h as u8];
        Ipv6Addr { inner: addr }
    }
}

impl AsInner<c::in_addr> for Ipv4Addr {
    fn as_inner(&self) -> &c::in_addr { &self.inner }
}


impl AsInner<c::in6_addr> for Ipv6Addr {
    fn as_inner(&self) -> &c::in6_addr { &self.inner }
}

// for &'a SocketAddr 这个是什么意思？
impl<'a> IntoInner<(*const c::sockaddr, c::socklen_t)> for &'a SocketAddr {
    fn into_inner(self) -> (*const c::sockaddr, c::socklen_t) {
        match *self {
            SocketAddr::V4(ref a) => {
                (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
            }
            SocketAddr::V6(ref a) => {
                (a as *const _ as *const _, mem::size_of_val(a) as c::socklen_t)
            }
        }
    }
}


pub enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
}

pub struct SocketAddrV4 {
    inner: c::sockaddr_in,
}

pub struct SocketAddrV6 {
    inner: c::sockaddr_in6,
}


impl SocketAddrV4 {
    pub fn new(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4 {
            inner: c::sockaddr_in {
                sin_family: c::AF_INET as c::sa_family_t,
                sin_port: hton(port),
                sin_addr: *ip.as_inner(),
                .. unsafe { mem::zeroed() }
            },
        }
    }
}

impl SocketAddrV6 {
    pub fn new(ip: Ipv6Addr, port:u16, flowinfo: u32, scope_id: u32) -> SocketAddrV6 {
        SocketAddrV6 {
            inner: c::sockaddr_in6 {
                sin6_family: c::AF_INET6 as c::sa_family_t,
                sin6_port: hton(port),
                sin6_addr: *ip.as_inner(),
                sin6_flowinfo: hton(flowinfo),
                sin6_scope_id: hton(scope_id),
                .. unsafe { mem::zeroed() }
            },
        }
    }
}

fn test_connect2() {
    let fd;
    unsafe {
        fd = c::socket(c::AF_INET, c::SOCK_STREAM, c::IPPROTO_TCP);
    }
    let a = Ipv4Addr::new(127, 0, 1, 3);
    //let a = Ipv4Addr::new(77, 88, 21, 11);
    let p = 12345;
    let s = SocketAddrV4::new(a, p);
    let servaddr = SocketAddr::V4(s);
    let (addrp, len) = servaddr.into_inner();
    unsafe {
        c::connect(fd, addrp, len);
    }
    
}


fn test_connect3() {
    let fd;
    unsafe {
        fd = c::socket(c::AF_INET6, c::SOCK_STREAM, c::IPPROTO_TCP);
    }
    let a = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    let p = 80;
    let s = SocketAddrV6::new(a, p, 0, 0);
    assert_eq!(c::AF_INET6 as c::sa_family_t, s.inner.sin6_family);
    println!("{0} c::AF_INET6 {1}", s.inner.sin6_family, c::AF_INET6);
    let servaddr = SocketAddr::V6(s);
    let (addrp, len) = servaddr.into_inner();
    unsafe {
        c::connect(fd, addrp, len);
    }
}

fn test_connect4(addr: SocketAddr) {
    let (addrp, len) = servaddr.into_inner();
    unsafe {
        c::conect(fd, addrp, len);
    }
}


fn main() {
    test_connect1();
    test_connect2();
    test_connect3();
    println!("中文你好！");
    println!("Hello, world!");
    let n: i32 = 300;
    println!("num is {0}", n.to_be());
    println!("num is {0}", i32::from_be(n.to_be()));
    
}
