use crate::PaymentRequest;

impl PaymentRequest {
    pub fn mock() -> Self {
        Self::new_address("0xcB3028d6120802148f03d6c884D6AD6A210Df62A")
    }
}
