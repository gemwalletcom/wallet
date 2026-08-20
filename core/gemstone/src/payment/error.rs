pub type PaymentError = payment::PaymentError;

#[uniffi::remote(Enum)]
pub enum PaymentError {
    NotSupported,
    PaymentNotFound,
    PaymentExpired,
    QuoteExpired,
    NoPaymentOptions,
    Rejected,
    InvalidRequest(String),
    Network(String),
}
