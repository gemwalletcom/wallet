pub type PaymentError = payment::PaymentError;

#[uniffi::remote(Enum)]
pub enum PaymentError {
    NotSupported,
    PaymentNotFound,
    PaymentExpired,
    QuoteExpired,
    NoPaymentOptions,
    UnsupportedAccounts,
    Rejected,
    RateLimited,
    InvalidRequest(String),
    Network(String),
}
