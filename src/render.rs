pub trait RenderPass {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self;
    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
    fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn update(&mut self) {}
}
