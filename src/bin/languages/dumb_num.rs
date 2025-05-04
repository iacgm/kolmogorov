use kolmogorov::*;

#[derive(Clone, Copy, Debug)]
pub struct DumbNum;

impl Language for DumbNum {
    type Semantics = OpaqueSemantics;
    fn context(&self) -> Context {
        super::NumLogic::new(0).context()
    }
}
