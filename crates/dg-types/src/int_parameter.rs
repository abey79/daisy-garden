//TODO: make that generic over int types?

pub trait IntParameter {
    async fn get(&mut self) -> i32;
}

impl IntParameter for i32 {
    async fn get(&mut self) -> i32 {
        *self
    }
}
