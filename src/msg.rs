use std::ops::Index;

#[derive(Debug)]
struct ContiguousView<'a> {
    slices: Vec<&'a [u8]>
}

impl<'a> ContiguousView<'a> {
    fn new(slices: Vec<&'a [u8]>) -> ContiguousView<'a> {
        ContiguousView {
            slices,
        }
    }
}

impl<'a> Index<usize> for ContiguousView<'a> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        let mut slice_start = 0;
        for s in &self.slices {
            if idx < slice_start + s.len() {
            println!("returning {:X?}[{:X?}]", s, idx - slice_start);
            return &s[idx - slice_start];
            } else {
                slice_start += s.len();
            }
        }
        // should generate out of bounds
        &self.slices[self.slices.len() - 1][idx]
    }
}

struct Request<'a> {
    /*
    0..2        len
    2           opcode
    3..len+3    opdata

    opdata for opcode 00 GET
    3..len+3    key

    opdata for opcode 01 PUT
    3..4                keylen
    4..keylen+4         key
    keylen+4..len+3     val
    */ 
    data: &'a [u8],
}

impl<'a> Request<'a> {

    fn new(data: &'a [u8]) -> Request<'a> {
        Request {
            data,
        }
    }

    fn len(&self) -> usize {
        // replace with usize::from_bytes()
        (256 * self.data[0] as u32 + self.data[1] as u32) as usize
    }

    fn op(&self) -> u8 {
        self.data[2]
    }

    fn key_len(&self) -> usize {
        if self.op() == 0x00 {
            self.len()
        } else {
            (256 * self.data[3] as u32 + self.data[4] as u32) as usize
        }
    }

    fn key(&self) -> &[u8] {
        if self.op() == 0x00 {
            &self.data[3..]
        } else {
            let keylen = self.key_len();
            &self.data[5..keylen+5]
        }
    }

    fn val(&self) -> &[u8] {
        let keylen = self.key_len();
        &self.data[keylen+5..]
    }
}

#[cfg(test)]
mod tests {

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
    fn test_view() {
        let s0 = "abc".as_bytes();
        let s1 = "xyz".as_bytes();
        let v = super::ContiguousView::new(vec![s0, s1]);
        println!("{:X?}", v);
        println!("{:X?}", v[0]);
        println!("{:X?}", v[5]);
    }

}