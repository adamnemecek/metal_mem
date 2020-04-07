

use metalgear::GPUVec;

//

fn main() {
    let dev = metal::Device::system_default().unwrap();

    let mut v = GPUVec::from_slice(&dev, &[1, 2, 3]);

    // dbg!("{}", v);

    // let new = [7, 8];
    // let u: Vec<_> = v.splice(..2, new.iter().cloned()).collect();
    // let result: Vec<_> = v.iter().cloned().collect();
    // let expected = vec![7, 8, 3];

    // for e in result {
    //     println!("{}", e);
    // }
    // println!("len {}", v.len());


    // let byte_capacity = self.byte_capacity();
    // let buffer1 = dev.new_buffer(
    //     64,
    //     metal::MTLResourceOptions::CPUCacheModeDefaultCache
    // );

    // let buffer2 = buffer1.clone();

    // println!("{}", format!("{:p}", buffer1.contents()));
    // println!("{}", format!("{:p}", buffer2.contents()));
}