pub trait FloatParameter {
    async fn get(&mut self) -> f32;
}

impl FloatParameter for f32 {
    async fn get(&mut self) -> f32 {
        *self
    }
}
