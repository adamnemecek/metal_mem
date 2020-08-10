/// this is necessary because of send + sync
struct Device {
    device: metal::Device,
}

impl Device {
    pub fn new(device: metal::Device) -> Self {
        Self { device }
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new(metal::Device::system_default().unwrap())
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

static mut DEVICE1: Option<Device> = None;

// lazy_static! {
//     static ref DEVICE: Device = Device::default();
// }
///
/// I hate this but it's the only way of allowing specs to work
/// with non-default GPUDevices
///
pub fn get_global_device() -> metal::Device {
    unsafe {
        if let Some(d) = &DEVICE1 {
            d.device.to_owned()
        } else {
            metal::Device::system_default().unwrap()
        }
    }
}

pub fn set_global_device(device: &metal::DeviceRef) {
    unsafe {
        DEVICE1 = Some(Device {
            device: device.to_owned(),
        });
    }
}
