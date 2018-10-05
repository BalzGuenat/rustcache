#[cfg(test)]

use std::str;

fn int_to_bytes(val: usize) -> [u8; 2] {
    [(val / 256) as u8, val as u8]
}

fn get(key: &str) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let mut vec = int_to_bytes(key_bytes.len()).to_vec();
    vec.push(0x00);
    vec.append(&mut key_bytes.to_vec());
    vec
}

fn put(key: &str, val: &str) -> Vec<u8> {
    let key_bytes = key.as_bytes();
    let val_bytes = val.as_bytes();
    let mut data = int_to_bytes(key_bytes.len()).to_vec();
    data.append(&mut key_bytes.to_vec());
    data.append(&mut val_bytes.to_vec());
    let mut vec: Vec<u8> = int_to_bytes(data.len()).to_vec();
    vec.push(0x01);
    vec.append(&mut data);
    vec
}

#[test]
fn test_get() {
    let rq = get("foo");
    println!("{:X?}", rq);
    let rq = super::Request::new(&rq);
    assert_eq!(3, rq.len());
    assert_eq!(0x00, rq.op());
    assert_eq!(3, rq.key_len());
    assert_eq!("foo", str::from_utf8(rq.key()).unwrap());
}

#[test]
fn test_put() {
    let rq = put("foo", "bar");
    println!("{:X?}", rq);
    let rq = super::Request::new(&rq);
    assert_eq!(8, rq.len());
    assert_eq!(0x01, rq.op());
    assert_eq!(3, rq.key_len());
    assert_eq!("foo", str::from_utf8(rq.key()).unwrap());
    assert_eq!("bar", str::from_utf8(rq.val()).unwrap());
}

#[test]
fn test_view_idx() {
    let s0 = "abc".as_bytes();
    let s1 = "xyz".as_bytes();
    let v = super::ContiguousView::new(vec![s0, s1]);
    println!("{:X?}", v);
    println!("{:X?}", v[0]);
    println!("{:X?}", v[5]);
    assert_eq!("a".as_bytes()[0], v[0]);
    assert_eq!("b".as_bytes()[0], v[1]);
    assert_eq!("c".as_bytes()[0], v[2]);
    assert_eq!("x".as_bytes()[0], v[3]);
    assert_eq!("y".as_bytes()[0], v[4]);
    assert_eq!("z".as_bytes()[0], v[5]);
}

#[test]
fn test_view_range() {
    let s0 = "abc".as_bytes();
    let s1 = "xyz".as_bytes();
    let v = super::ContiguousView::new(vec![s0, s1]);
    println!("{:X?}", v);
    assert_eq!("abc".as_bytes(), &v[0..3]);
    assert_eq!("xyz".as_bytes(), &v[3..6]);
    assert_eq!("ab".as_bytes(), &v[0..2]);
    assert_eq!("yz".as_bytes(), &v[4..6]);
    assert_eq!("cx".as_bytes(), &v[2..4]);
    println!("{:X?}", &v[2..4]);
    assert_eq!("abcxyz".as_bytes(), &v[0..6]);
}

#[test]
fn test_view_iter() {
    let s0 = "abc".as_bytes();
    let s1 = "xyz".as_bytes();
    let v = super::ContiguousView::new(vec![s0, s1]);
    println!("{:X?}", v);
    let mut vec = Vec::new();
    for b in v {
        vec.push(b);
    }
    println!("{:?}", vec);
    assert_eq!("abcxyz".as_bytes(), &vec[..]);
}

#[test]
fn test_view_slice_iter() {
    let s0 = "abc".as_bytes();
    let s1 = "xyz".as_bytes();
    let v = super::ContiguousView::new(vec![s0, s1]);
    println!("{:X?}", v);
    let mut vec = Vec::new();
    for b in v[0..v.len()] {
        vec.push(b);
    }
    println!("{:?}", vec);
    assert_eq!("abcxyz".as_bytes(), &vec[..]);
}

#[test]
fn test_view_len() {
    let s0 = "abc".as_bytes();
    let s1 = "xyz".as_bytes();
    let v = super::ContiguousView::new(vec![s0, s1]);
    assert_eq!(6, v.len());
}
