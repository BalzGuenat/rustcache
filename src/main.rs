#![allow(unused_imports)]

use std::str;
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream, SocketAddr};

use std::collections::HashMap;

const DEFAULT_PORT: u16 = 8181;
const OP_GET: u8 = 0x00;
const OP_PUT: u8 = 0x01;
const OP_DEL: u8 = 0x02;

// static mut THE_MAP: HashMap<[u8], [u8]> = HashMap::new();

#[derive(Debug)]
struct RcPackage {
    // data
    header: [u8; 3],
    data: Vec<u8>,
    // derived but stored anyway
    len: usize,
    op: u8,
}

impl RcPackage {
    fn val(&self) -> &[u8] {
        let keylen = self.data[0] as usize;
        return &self.data[keylen+1..];
    }
}

fn build_pkg(header: [u8; 3], data: Vec<u8>) -> RcPackage {
    RcPackage {
        header,
        data,
        len: (256 * header[0] as u32 + header[1] as u32) as usize,
        op: header[2],
    }
}

fn main() -> io::Result<()> {
    let port = DEFAULT_PORT;

    let mut map = HashMap::new();

    println!("Listening on port {:}", port);
    let listener = TcpListener::bind(("localhost", port))?;

    for stream in listener.incoming() {
        handle(&mut map, stream?).expect("Error.");
    }

    Ok(())
}

fn handle(map: &mut HashMap<Vec<u8>, RcPackage>, mut stream: TcpStream) -> io::Result<()> {
    println!("Accepted connection from {:?}", stream.peer_addr().expect("Error."));
    let mut buf: [u8; 3] = [0; 3];
    let bytes_read = stream.read(&mut buf)?;
    if bytes_read < 3 {
        return Err(Error::new(ErrorKind::Other, "Invalid message."));
    }
    println!("rcv ({:}): {:X?}", bytes_read, buf);
    let len = (256 * buf[0] as u32 + buf[1] as u32) as usize;
    println!("len = {:?}", len);
    let mut msgbuf = Vec::with_capacity(len);
    unsafe { msgbuf.set_len(len) };
    let bytes_read = stream.read(&mut msgbuf[..])?;
    unsafe { msgbuf.set_len(bytes_read) };
    println!("rcv ({:}): {:X?}", bytes_read, msgbuf);

    let pkg = build_pkg(buf, msgbuf);

    match buf[2] {
        OP_GET => handle_get(map, stream, pkg),
        OP_PUT => handle_put(map, stream, pkg),
        // OP_DEL => handle_del(stream, &msgbuf[..]),
        _ => return Err(Error::new(ErrorKind::Other, format!("Invalid OP: {:X?}", buf[2]))),
    }
}

fn handle_get(map: &HashMap<Vec<u8>, RcPackage>, mut stream: TcpStream, pkg: RcPackage) -> io::Result<()> {
    println!("GET");

    let key = &pkg.data[..];
    match map.get(&key.to_vec()) {
        Some(valpkg) => {
            println!("snd {} -> {}", desc(key), desc(valpkg.val()));
            stream.write(valpkg.val())?;
            stream.flush()?;
            Ok(())
        },
        None => {
            println!("unmapped key {:?}", desc(key));
            Ok(())
        },
    }
}

fn handle_put(map: &mut HashMap<Vec<u8>, RcPackage>, stream: TcpStream, pkg: RcPackage) -> io::Result<()> {
    println!("PUT");

    let keylen = pkg.data[0] as usize;
    let mut key = Vec::with_capacity(keylen);
    unsafe { key.set_len(keylen) };
    key.copy_from_slice(&pkg.data[1..1+keylen]);

    // let msgbuf = &pkg.data[..];
    // let key = &msgbuf[1..keylen+1];
    // let val = &pkg.data[keylen+1..];

    let insertstr = format!("inserted {:X?} -> {:X?}", desc(&key[..]), desc(&pkg.data[keylen+1..]));
    let updatestr = format!("updated {:X?} -> {:X?}", desc(&key[..]), desc(&pkg.data[keylen+1..]));

    match map.insert(key, pkg) {
        None => println!("{}", insertstr),
        _ => println!("{}", updatestr),
    }

    // stream.shutdown(Shutdown::Both);

    Ok(())
}

fn desc(bytes: &[u8]) -> String {
    match str::from_utf8(bytes) {
        Ok(str) => str.to_string(),
        Err(_) => format!("{:X?}", bytes),
    }
}
