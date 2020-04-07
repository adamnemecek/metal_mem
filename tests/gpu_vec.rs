
// mod tests {
    // use super::*;
use metalgear::GPUVec;

#[test]
fn test_from_slice() {
    let dev = metal::Device::system_default().unwrap();
    let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    println!("capacity: {}", vec.capacity());

    for e in 0..vec.len() {
        assert!(vec[e] == e);
    }
}

#[test]
fn test_index() {
    let dev = metal::Device::system_default().unwrap();
    let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 3);
    assert!(vec[4] == 4);
    assert!(vec[5] == 5);
    assert!(vec[6] == 6);
}

#[test]
fn test_index_mut() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

    vec[0] = 8;
    vec[1] = 8;
    vec[2] = 8;
    vec[3] = 8;
    vec[4] = 8;
    vec[5] = 8;
    vec[6] = 8;

    assert!(vec[0] == 8);
    assert!(vec[1] == 8);
    assert!(vec[2] == 8);
    assert!(vec[3] == 8);
    assert!(vec[4] == 8);
    assert!(vec[5] == 8);
    assert!(vec[6] == 8);
}

#[test]
fn test_extend() {
    let dev = metal::Device::system_default().unwrap();
    let v = vec![0,1,2,3,4,5,6];
    let mut vec = GPUVec::from_slice(&dev, &v);
    vec.extend(v.into_iter());

    assert!(vec.len() == 14);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 3);
    assert!(vec[4] == 4);
    assert!(vec[5] == 5);
    assert!(vec[6] == 6);

    assert!(vec[7] == 0);
    assert!(vec[8] == 1);
    assert!(vec[9] == 2);
    assert!(vec[10] == 3);
    assert!(vec[11] == 4);
    assert!(vec[12] == 5);
    assert!(vec[13] == 6);
}

#[test]
fn test_extend_from_slice() {
    let dev = metal::Device::system_default().unwrap();
    let v = vec![0,1,2,3,4,5,6];
    let mut vec = GPUVec::from_slice(&dev, &v);

    vec.extend_from_slice(&v);

    assert!(vec.len() == 14);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 3);
    assert!(vec[4] == 4);
    assert!(vec[5] == 5);
    assert!(vec[6] == 6);

    assert!(vec[7] == 0);
    assert!(vec[8] == 1);
    assert!(vec[9] == 2);
    assert!(vec[10] == 3);
    assert!(vec[11] == 4);
    assert!(vec[12] == 5);
    assert!(vec[13] == 6);
}

#[test]
fn test_push() {
    let dev = metal::Device::system_default().unwrap();

    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    assert!(vec.len() == 7);

    vec.push(7);

    assert!(vec.len() == 8);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 3);
    assert!(vec[4] == 4);
    assert!(vec[5] == 5);
    assert!(vec[6] == 6);
    assert!(vec[7] == 7);
}

#[test]
fn test_insert() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,4,5,6]);
    vec.insert(3, 3);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 3);
    assert!(vec[4] == 4);
    assert!(vec[5] == 5);
    assert!(vec[6] == 6);
}

#[test]
fn test_truncate() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    vec.truncate(3);

    assert!(vec.len() == 3);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec.get(3) == None);
}

#[test]
fn test_remove() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    vec.remove(3);

    assert!(vec[0] == 0);
    assert!(vec[1] == 1);
    assert!(vec[2] == 2);
    assert!(vec[3] == 4);
    assert!(vec[4] == 5);
    assert!(vec[5] == 6);
}

#[test]
fn test_iter() {
    let dev = metal::Device::system_default().unwrap();
    let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);
    let mut v1_iter = v1.iter();

    // iter() returns an iterator of slices.
    assert_eq!(v1_iter.next(), Some(&1));
    assert_eq!(v1_iter.next(), Some(&2));
    assert_eq!(v1_iter.next(), Some(&3));
    assert_eq!(v1_iter.next(), None);
}

#[test]
fn test_into_iter() {
    let dev = metal::Device::system_default().unwrap();
    let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);
    let mut v1_iter = v1.into_iter();

    // into_iter() returns an iterator from a value.
    assert_eq!(v1_iter.next(), Some(1));
    assert_eq!(v1_iter.next(), Some(2));
    assert_eq!(v1_iter.next(), Some(3));
    assert_eq!(v1_iter.next(), None);
}

#[test]
fn test_iter_mut() {
    let dev = metal::Device::system_default().unwrap();
    let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);

    let mut v1_iter = v1.iter_mut();

    // iter_mut() returns an iterator that allows modifying each value.
    assert_eq!(v1_iter.next(), Some(&mut 1));
    assert_eq!(v1_iter.next(), Some(&mut 2));
    assert_eq!(v1_iter.next(), Some(&mut 3));
    assert_eq!(v1_iter.next(), None);
}

#[test]
fn test_retain() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    vec.retain(|x| x % 2 == 0);
    assert!(vec.len() == 4);

    assert!(vec[0] == 0);
    assert!(vec[1] == 2);
    assert!(vec[2] == 4);
    assert!(vec[3] == 6);
}

#[test]
fn test_eq() {
    let dev = metal::Device::system_default().unwrap();

    let mut a = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    let mut b = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    let mut c = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,7]);

    assert!(a == b);
    assert!(b != c);
}

#[test]
fn test_swap_remove() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

    let expected = GPUVec::from_slice(&dev, &[0,1,2,6,4,5]);

    let res = vec.swap_remove(3);

    assert!(res == 3);
    assert!(expected == vec);
}

#[test]
fn test_clear() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    vec.clear();

    assert!(vec.is_empty());
}

#[test]
fn test_clone() {
    let dev = metal::Device::system_default().unwrap();
    let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
    let copy = vec.clone();
    assert!(vec.len() == copy.len());
    assert!(vec.capacity() == copy.capacity());

    assert!(copy[0] == 0);
    assert!(copy[1] == 1);
    assert!(copy[2] == 2);
    assert!(copy[3] == 3);
    assert!(copy[4] == 4);
    assert!(copy[5] == 5);
    assert!(copy[6] == 6);
    assert!(vec.as_ptr() != copy.as_ptr());
}

#[test]
fn test_drain() {
    let dev = metal::Device::system_default().unwrap();
    let mut v: GPUVec<u32> = GPUVec::from_slice(&dev, &[1, 2, 3]);
    let u: Vec<_> = v.drain(1..).collect();
    assert!(v[..] == [1][..]);
    assert!(u[..] == [2,3][..]);
}

// #[test]
// fn test_drain_items() {
//     let mut vec = vec![1, 2, 3];
//     let mut vec2 = vec![];
//     for i in vec.drain(..) {
//         vec2.push(i);
//     }
//     assert_eq!(vec, []);
//     assert_eq!(vec2, [1, 2, 3]);
// }

// #[test]
// fn test_drain_items_reverse() {
//     let mut vec = vec![1, 2, 3];
//     let mut vec2 = vec![];
//     for i in vec.drain(..).rev() {
//         vec2.push(i);
//     }
//     assert_eq!(vec, []);
//     assert_eq!(vec2, [3, 2, 1]);
// }

// #[test]
// fn test_drain_items_zero_sized() {
//     let mut vec = vec![(), (), ()];
//     let mut vec2 = vec![];
//     for i in vec.drain(..) {
//         vec2.push(i);
//     }
//     assert_eq!(vec, []);
//     assert_eq!(vec2, [(), (), ()]);
// }

// #[test]
// fn test_drain_range() {
//     let mut v = vec![1, 2, 3, 4, 5];
//     for _ in v.drain(4..) {}
//     assert_eq!(v, &[1, 2, 3, 4]);

//     let mut v: Vec<_> = (1..6).map(|x| x.to_string()).collect();
//     for _ in v.drain(1..4) {}
//     assert_eq!(v, &[1.to_string(), 5.to_string()]);

//     let mut v: Vec<_> = (1..6).map(|x| x.to_string()).collect();
//     for _ in v.drain(1..4).rev() {}
//     assert_eq!(v, &[1.to_string(), 5.to_string()]);

//     let mut v: Vec<_> = vec![(); 5];
//     for _ in v.drain(1..4).rev() {}
//     assert_eq!(v, &[(), ()]);
// }

// #[test]
// fn test_drain_inclusive_range() {
//     let mut v = vec!['a', 'b', 'c', 'd', 'e'];
//     for _ in v.drain(1..=3) {}
//     assert_eq!(v, &['a', 'e']);

//     let mut v: Vec<_> = (0..=5).map(|x| x.to_string()).collect();
//     for _ in v.drain(1..=5) {}
//     assert_eq!(v, &["0".to_string()]);

//     let mut v: Vec<String> = (0..=5).map(|x| x.to_string()).collect();
//     for _ in v.drain(0..=5) {}
//     assert_eq!(v, Vec::<String>::new());

//     let mut v: Vec<_> = (0..=5).map(|x| x.to_string()).collect();
//     for _ in v.drain(0..=3) {}
//     assert_eq!(v, &["4".to_string(), "5".to_string()]);

//     let mut v: Vec<_> = (0..=1).map(|x| x.to_string()).collect();
//     for _ in v.drain(..=0) {}
//     assert_eq!(v, &["1".to_string()]);
// }

#[test]
fn test_drain_filter() {
    // let dev = metal::Device::system_default().unwrap();
    // let mut v = GPUVec::from_slice(&dev, &[1, 2, 3]);

    // let new = [7, 8];
    // let u: Vec<_> = v.splice(..2, new.iter().cloned()).collect();

    // assert!(v.iter().eq([7,8,3].iter()));
    // assert!(u.iter().eq([1,2].iter()));
    // // let expected = vec![7, 8, 3];
    // // dbg!("{}", &result);
    // // assert!(result == expected);


    // // assert_eq!(u, &[1, 2]);
}

/// taken from rustdoc test for splice
#[test]
fn test_splice() {
    let dev = metal::Device::system_default().unwrap();
    let mut v = GPUVec::from_slice(&dev, &[1, 2, 3]);

    let new = [7, 8];
    let u: Vec<_> = v.splice(..2, new.iter().cloned()).collect();

    assert!(v[..] == [7,8,3][..]);
    assert!(u[..] == [1,2][..]);
}

// #[test]
// fn test_splice_inclusive_range() {
//     let mut v = vec![1, 2, 3, 4, 5];
//     let a = [10, 11, 12];
//     let t1: Vec<_> = v.splice(2..=3, a.iter().cloned()).collect();
//     assert_eq!(v, &[1, 2, 10, 11, 12, 5]);
//     assert_eq!(t1, &[3, 4]);
//     let t2: Vec<_> = v.splice(1..=2, Some(20)).collect();
//     assert_eq!(v, &[1, 20, 11, 12, 5]);
//     assert_eq!(t2, &[2, 10]);
// }

#[test]
fn test_splice_inclusive_range() {
    let dev = metal::Device::system_default().unwrap();
    let mut v = GPUVec::from_slice(&dev, &[1, 2, 3, 4, 5, 6]);
    let a = [10, 11, 12];
    let t1: Vec<_> = v.splice(2..=3, a.iter().cloned()).collect();
    dbg!("{}", v.to_vec());

    assert!(v[..] == [1, 2, 10, 11, 12, 5, 6][..]);
    assert!(t1[..] == [3,4][..]);

    let t2: Vec<_> = v.splice(1..=2, Some(20)).collect();
    assert!(v[..] == [1, 20, 11, 12, 5, 6][..]);
    assert!(t2[..] == [2,10][..]);
}


// #[test]
// #[should_panic]
// fn test_splice_out_of_bounds() {
//     let mut v = vec![1, 2, 3, 4, 5];
//     let a = [10, 11, 12];
//     v.splice(5..6, a.iter().cloned());
// }

// #[test]
// #[should_panic]
// fn test_splice_inclusive_out_of_bounds() {
//     let mut v = vec![1, 2, 3, 4, 5];
//     let a = [10, 11, 12];
//     v.splice(5..=5, a.iter().cloned());
// }

// #[test]
// fn test_splice_items_zero_sized() {
//     let mut vec = vec![(), (), ()];
//     let vec2 = vec![];
//     let t: Vec<_> = vec.splice(1..2, vec2.iter().cloned()).collect();
//     assert_eq!(vec, &[(), ()]);
//     assert_eq!(t, &[()]);
// }

#[test]
fn test_splice_unbounded() {
    let dev = metal::Device::system_default().unwrap();
    let mut vec = GPUVec::from_slice(&dev, &[1, 2, 3, 4, 5]);
    let t: Vec<_> = vec.splice(.., None).collect();
    assert!(vec[..]  == [][..]);
    assert!(t[..] == [1, 2, 3, 4, 5][..]);
}

// #[test]
// fn test_splice_forget() {
//     let mut v = vec![1, 2, 3, 4, 5];
//     let a = [10, 11, 12];
//     std::mem::forget(v.splice(2..4, a.iter().cloned()));
//     assert_eq!(v, &[1, 2]);
// }



