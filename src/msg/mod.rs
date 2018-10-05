use std::ops::{Index, Range};
use std::slice;
use std::iter::IntoIterator;

mod test;

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

    fn iter(&self) -> ContiguousViewIt {
        // self.slices[0].iter()
        ContiguousViewIt {
            view: self,
            next_idx: 0,
            end_idx: self.len(),
        }
    }

    fn len(&self) -> usize {
        let mut sum = 0;
        for s in &self.slices {
            sum += s.len();
        }
        sum
    }
}

impl<'a> Index<usize> for ContiguousView<'a> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        let mut slice_start = 0;
        for s in &self.slices {
            if idx < slice_start + s.len() {
            return &s[idx - slice_start];
            } else {
                slice_start += s.len();
            }
        }
        // should generate out of bounds
        &self.slices[self.slices.len() - 1][idx]
    }
}

impl<'a> Index<Range<usize>> for ContiguousView<'a> {
    type Output = ContiguousViewIt<'a>;

    fn index(&self, idx: Range<usize>) -> &Self::Output {
        &ContiguousViewIt<'a> {
            view: self,
            next_idx: idx.start,
            end_idx: idx.end,
        }
        // let start = idx.start;
        // let end = idx.end;
        // let mut slice_start = 0;
        // for s in &self.slices {
        //     if start < slice_start + s.len() {
        //     return &s[start - slice_start..end - slice_start];
        //     } else {
        //         slice_start += s.len();
        //     }
        // }
        // // should generate out of bounds
        // &self.slices[self.slices.len() - 1][start..end]
    }
}

impl<'a> IntoIterator for ContiguousView<'a> {
    type Item = u8;
    type IntoIter = ContiguousViewOwningIt<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ContiguousViewOwningIt {
            view: self,
            next_idx: 0,
            end_idx: self.len(),
        }
    }
}

struct ContiguousViewIt<'a> {
    view: &'a ContiguousView<'a>,
    // slice_it: Iterator<Item=u8>,
    next_idx: usize,
    end_idx: usize,
}

struct ContiguousViewOwningIt<'a> {
    view: ContiguousView<'a>,
    // slice_it: Iterator<Item=u8>,
    next_idx: usize,
}

impl<'a> Iterator for ContiguousViewIt<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let next = if self.next_idx >= self.end_idx {
            None
        } else {
            Some(self.view[self.next_idx])
        };
        self.next_idx += 1;
        next
    }
}

impl<'a> Iterator for ContiguousViewOwningIt<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let next = if self.next_idx >= self.end_idx {
            None
        } else {
            Some(self.view[self.next_idx])
        };
        self.next_idx += 1;
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for ContiguousViewOwningIt<'a> {
    fn len(&self) -> usize {
        self.view.len() - self.next_idx
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
