
// mod tests {
use metalgear::GPUVar;
#[test]
fn test_var() {
    let dev = metal::Device::system_default().unwrap();
    let mut var = GPUVar::new(&dev, 10);
    assert!(var.value() == 10);

    var.set_value(20);
    assert!(var.value() == 20);
}
// }