
// mod tests {
use metalgear::GPUVar;
#[test]
fn test_var() {
    let dev = metal::Device::system_default().unwrap();
    let mut var = GPUVar::new(&dev, 10);
    assert!(*var == 10);

    *var = 20;
    assert!(*var == 20);
}
// }