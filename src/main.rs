#![allow(unused_imports)]

extern crate threadpool;

use std::str;
use std::io;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream, SocketAddr};

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::thread;

use threadpool::ThreadPool;

const DEFAULT_PORT: u16 = 8181;
const NUM_WORKERS: usize = 4;
const MAX_WAITING: usize = 16;

const OP_GET: u8 = 0x00;
const OP_PUT: u8 = 0x01;
const OP_DEL: u8 = 0x02;

type MapType = Arc<Mutex<HashMap<Vec<u8>, RcPackage>>>;

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
    fn val_len(&self) -> usize {
        let keylen = self.data[0] as usize;
        let val_len = self.data.len() - keylen - 1;
        assert!(val_len == self.val().len());
        val_len
    }

    fn val(&self) -> &[u8] {
        let keylen = self.data[0] as usize;
        &self.data[keylen+1..]
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

    let map = HashMap::new();
    let maplock = Arc::new(Mutex::new(map));
    // let pool = ThreadPool::new(NUM_WORKERS);

    let (tx, rx) = mpsc::sync_channel(MAX_WAITING);
    let queue_front = Arc::new(Mutex::new(rx));
    let queue_back = Arc::new(Mutex::new(tx));
    let mut workers = Vec::with_capacity(NUM_WORKERS);
    for i in 0..NUM_WORKERS {
        let maplock = maplock.clone();
        let queue_front = queue_front.clone();
        let queue_back = queue_back.clone();
        let thread_builder = thread::Builder::new().name(format!("worker-{}", i));
        let worker = thread_builder.spawn(move || {
            loop {
                let rx = queue_front.lock();
                let stream = rx.unwrap().recv().unwrap();
                match handle_conn(maplock.clone(), &stream) {
                    Ok(()) => {
                        let tx = queue_back.lock();
                        tx.unwrap().send(stream).expect("Failed to queue stream.");
                    }
                    Err(e) => match e.kind() {
                        ErrorKind::ConnectionAborted => (),
                        _ => return Err(e).expect("Error.")
                    }
                }
            }
        }).expect("failed to create worker thread.");
        workers.push(worker);
    }

    println!("Listening on port {:}", port);
    let listener = TcpListener::bind(("localhost", port))?;

    for stream in listener.incoming() {
        // handle(&mut map, stream?).expect("Error.");
        // tx.send(stream?).unwrap();
        if let Ok(stream) = stream {
            {
                let stream = &stream;
                let peer = stream.peer_addr()?;
                println!("Accepted connection from {:?}", peer);
            }
            let tx = queue_back.lock();
            tx.unwrap().send(stream).expect("Failed to queue stream.");
            // let maplock = Arc::clone(&maplock);
            // pool.execute(move || {
            //     match handle_conn(maplock, stream) {
            //         _ => println!("connection closed"),
            //     };
            // });
        }
    }

    Ok(())
}

fn handle_conn(map: MapType, mut stream: &TcpStream) -> io::Result<()> {
    let mut buf: [u8; 3] = [0; 3];
    let bytes_read = stream.read(&mut buf)?;
    if bytes_read == 0 {
        println!("no bytes read.");
        return Err(Error::new(ErrorKind::ConnectionAborted, "EOF."));
    } else if bytes_read < 3 {
        return Err(Error::new(ErrorKind::Other, "Incomplete message."));
    }
    println!("{:?}: start parsing msg from {:?}", thread::current().name().unwrap_or("unknown thread"), stream.peer_addr()?);
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
        OP_GET => handle_get(&map, &stream, pkg)?,
        OP_PUT => handle_put(&map, &stream, pkg)?,
        // OP_DEL => handle_del(stream, &msgbuf[..]),
        _ => return Err(Error::new(ErrorKind::Other, format!("Invalid OP: {:X?}", buf[2]))),
    }
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

fn handle_get(maplock: &MapType, mut stream: &TcpStream, pkg: RcPackage) -> io::Result<()> {
    println!("GET");

    let key = &pkg.data[..];
    let map = maplock.lock().unwrap();
    match map.get(&key.to_vec()) {
        Some(valpkg) => {
            let val = valpkg.val();
            println!("snd {} -> {}", desc(key), desc(val));
            let val_len = &[valpkg.val_len() as u8];
            println!("ie {:X?}{:X?}", &val_len, &val);
            stream.write(val_len)?;
            stream.write(val)?;
            Ok(())
        },
        None => {
            println!("unmapped key {:?}", desc(key));
            Ok(())
        },
    }
}

fn handle_put(maplock: &MapType, mut stream: &TcpStream, pkg: RcPackage) -> io::Result<()> {
    println!("PUT");

    let keylen = pkg.data[0] as usize;
    // let mut key = Vec::with_capacity(keylen);
    // unsafe { key.set_len(keylen) };
    // key.copy_from_slice(&pkg.data[1..1+keylen]);
    let  key = pkg.data[1..1+keylen].to_vec();

    // let msgbuf = &pkg.data[..];
    // let key = &msgbuf[1..keylen+1];
    // let val = &pkg.data[keylen+1..];

    let insertstr = format!("inserted {:X?} -> {:X?}", desc(&key[..]), desc(&pkg.data[keylen+1..]));
    let updatestr = format!("updated {:X?} -> {:X?}", desc(&key[..]), desc(&pkg.data[keylen+1..]));

    let mut map = maplock.lock().unwrap();
    match map.insert(key, pkg) {
        None => println!("{}", insertstr),
        _ => println!("{}", updatestr),
    }

    stream.write(&[0x00])?;

    // stream.shutdown(Shutdown::Both);

    Ok(())
}

fn desc(bytes: &[u8]) -> String {
    match str::from_utf8(bytes) {
        Ok(str) => str.to_string(),
        Err(_) => format!("{:X?}", bytes),
    }
}
