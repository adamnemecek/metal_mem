use metalgear::{round_up, MemAlign};

#[test]
fn test_roundup() {
    // assert!(round_up(0, 4096) == 4096);
    // println!("{}", round_up(0, 4096));
    assert!(round_up(1, 4096) == 4096);
    assert!(round_up(4095, 4096) == 4096);
    assert!(round_up(4096, 4096) == 4096);
    assert!(round_up(4097, 4096) == 2 * 4096);
    assert!(round_up(2 * 4096 + 1, 4096) == 3 * 4096);
}

#[test]
fn test_paged_alloc() {
    #[repr(C)]
    struct TestStruct {
        data: [u8; 18],
    }

    let element_size: usize = std::mem::size_of::<TestStruct>();
    assert!(element_size == 18);
    let count = 10;
    // let page_size = 4096;
    let alloc = MemAlign::<TestStruct>::new(count);

    // println!("{}", alloc.is_valid());

    // dbg!("{}", alloc);
}
