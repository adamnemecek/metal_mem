

use metalgear::GPUVec;

fn main() {
    let dev = metal::Device::system_default().unwrap();
    let v: Vec<usize> = vec![0,1,2,3,4,5,6];
    let mut vec = GPUVec::from_iter(&dev, &v);

    let f = vec.first().unwrap();

    println!("{}", f);
    for e in vec.into_iter() {
        println!("{}", e);
    }
}