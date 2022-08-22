pub mod backend_d3d12;

pub trait Backend: Sized {
    type Instance: Instance<Self>;
}

pub trait Instance<B: Backend> {
    fn enumerate_adapters(&self);
}

pub trait Adapter<B: Backend> {}

pub trait Device<B: Backend> {}
