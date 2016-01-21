extern crate libc as c;
use std::mem as mem;
trait NetInt {
    fn from_be(i: Self) -> Self;
    fn to_be(&self) -> Self;
}
macro_rules! doit {
    ($($t:ident)*) => ($(impl NetInt for $t {
        fn from_be(i: Self) -> Self { <$t>::from_be(i) }
        fn to_be(&self) -> Self { <$t>::to_be(*self) }
    })*)
}
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
        c::connect(fd, (&servaddr as *const _ as *const c::sockaddr), mem::size_of_val(&servaddr) as c::socklen_t);
    }    
}

pub trait AsInner<Inner: ?Sized> {
    fn as_inner(&self) -> &Inner;
}

pub struct Ipv4Addr {
    inner: c::in_addr,
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

impl AsInner<c::in_addr> for Ipv4Addr {
    fn as_inner(&self) -> &c::in_addr { &self.inner }
}

pub enum SocketAddr {
    V4(SocketAddrV4),
    //V6(SocketAddrV6),
}

pub struct SocketAddrV4 {
    inner: c::sockaddr_in,
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

fn test_connect2() {
    let fd;
    unsafe {
        fd = c::socket(c::AF_INET, c::SOCK_STREAM, c::IPPROTO_TCP);
    }
    let a = Ipv4Addr::new(77, 88, 21, 11);
    let p = 12345;
    let s = SocketAddrV4::new(a, p);
    let servaddr = SocketAddr::V4(s);
    unsafe {
        c::connect(fd, (&servaddr as *const _ as *const c::sockaddr), mem::size_of_val(&servaddr) as c::socklen_t);
    }
    
}

fn main() {
    test_connect1();
    test_connect2();
    println!("中文你好！");
    println!("Hello, world!");
    let n: i32 = 300;
    println!("num is {0}", n.to_be());
    println!("num is {0}", i32::from_be(n.to_be()));
    
}
