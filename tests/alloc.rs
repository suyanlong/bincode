#![cfg(feature = "alloc")]

extern crate alloc;

mod utils;

use alloc::borrow::Cow;
use alloc::collections::*;
use alloc::rc::Rc;
#[cfg(feature = "atomic")]
use alloc::sync::Arc;
use bincode::config::Configuration;
use utils::{the_same, the_same_with_comparer};

struct Foo {
    pub a: u32,
    pub b: u32,
}

impl bincode::enc::Encode for Foo {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        mut encoder: E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.a.encode(&mut encoder)?;
        self.b.encode(&mut encoder)?;
        Ok(())
    }
}

impl bincode::Decode for Foo {
    fn decode<D: bincode::de::Decoder>(
        mut decoder: D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            a: bincode::Decode::decode(&mut decoder)?,
            b: bincode::Decode::decode(&mut decoder)?,
        })
    }
}

#[test]
fn test_vec() {
    let vec = bincode::encode_to_vec(Foo { a: 5, b: 10 }, Configuration::standard()).unwrap();
    assert_eq!(vec, &[5, 10]);

    let (foo, len): (Foo, usize) =
        bincode::decode_from_slice(&vec, Configuration::standard()).unwrap();
    assert_eq!(foo.a, 5);
    assert_eq!(foo.b, 10);
    assert_eq!(len, 2);
}

#[test]
fn test_alloc_commons() {
    the_same::<Vec<u32>>(vec![1, 2, 3, 4, 5]);
    the_same(String::from("Hello world"));
    the_same(Box::<u32>::new(5));
    the_same(Box::<[u32]>::from(vec![1, 2, 3, 4, 5]));
    the_same(Cow::<u32>::Owned(5));
    the_same(Cow::<u32>::Borrowed(&5));
    the_same(Rc::<u32>::new(5));
    #[cfg(feature = "atomic")]
    the_same(Arc::<u32>::new(5));
    the_same_with_comparer(
        {
            let mut map = BinaryHeap::<u32>::new();
            map.push(1);
            map.push(2);
            map.push(3);
            map.push(4);
            map.push(5);
            map
        },
        |a, b| a.into_iter().collect::<Vec<_>>() == b.into_iter().collect::<Vec<_>>(),
    );
    the_same({
        let mut map = BTreeMap::<u32, i32>::new();
        map.insert(5, -5);
        map
    });
    the_same({
        let mut set = BTreeSet::<u32>::new();
        set.insert(5);
        set
    });
    the_same({
        let mut set = VecDeque::<u32>::new();
        set.push_back(15);
        set.push_front(5);
        set
    });
}

#[test]
fn test_container_limits() {
    use bincode::{error::DecodeError, Decode};

    const DECODE_LIMIT: usize = 100_000;

    // for this test we'll create a malformed package of a lot of bytes
    let test_cases = &[
        // u64::max_value(), should overflow
        bincode::encode_to_vec(u32::max_value(), Configuration::standard()).unwrap(),
        // A high value which doesn't overflow, but exceeds the decode limit
        bincode::encode_to_vec(DECODE_LIMIT as u32, Configuration::standard()).unwrap(),
    ];

    fn validate_fail<T: Decode + core::fmt::Debug>(slice: &[u8]) {
        let result = bincode::decode_from_slice::<T, _>(
            slice,
            Configuration::standard().with_limit::<DECODE_LIMIT>(),
        );

        assert_eq!(result.unwrap_err(), DecodeError::LimitExceeded);
    }

    for slice in test_cases {
        validate_fail::<BinaryHeap<i32>>(slice);
        validate_fail::<BTreeMap<i32, i32>>(slice);
        validate_fail::<BTreeSet<i32>>(slice);
        validate_fail::<VecDeque<i32>>(slice);
        validate_fail::<Vec<i32>>(slice);
        validate_fail::<String>(slice);
        validate_fail::<Box<[u8]>>(slice);
        #[cfg(feature = "std")]
        {
            validate_fail::<std::collections::HashMap<i32, i32>>(slice);
        }
    }
}
