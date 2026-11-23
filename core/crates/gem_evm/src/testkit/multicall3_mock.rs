use crate::multicall3::IMulticall3;

impl IMulticall3::Result {
    pub fn mock(return_data: Vec<u8>) -> Self {
        Self {
            success: true,
            returnData: return_data.into(),
        }
    }
}
