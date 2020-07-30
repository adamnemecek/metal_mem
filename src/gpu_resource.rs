pub trait GPUResource {
    type Device;
    fn device(&self) -> &Self::Device;
    fn set_device(&mut self, device: &Self::Device);
}
