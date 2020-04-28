
#[macro_use]
extern crate metalgear;
// mod tests {
use metalgear::GPUVar;

#[test]
fn test_var_mac() {
    let var = gpuvar![1];
    assert!(*var == 1);
}

#[test]
fn test_var() {
    let dev = metal::Device::system_default().unwrap();
    let mut var = GPUVar::with_value(&dev, 10);
    assert!(*var == 10);

    *var = 20;
    assert!(*var == 20);
}
// }