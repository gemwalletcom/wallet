use primitives::{ConfigResponse, ConfigVersions, FiatTransactionData, InAppNotification, StreamEvent, StreamMessage, SupportMessage, SupportMessageInput, SupportTyping};

macro_rules! json_bridge {
    ($($type:ident),* $(,)?) => {
        $(
            uniffi::custom_type!($type, String, {
                remote,
                lower: |value| match serde_json::to_string(&value) {
                    Ok(json) => json,
                    Err(error) => {
                        debug_assert!(false, concat!("failed to serialize ", stringify!($type), ": {}"), error);
                        String::new()
                    }
                },
                try_lift: |value| serde_json::from_str(&value).map_err(|error| {
                    uniffi::deps::anyhow::Error::msg(format!(concat!("invalid ", stringify!($type), ": {}"), error))
                }),
            });
        )*
    };
}

json_bridge!(
    ConfigResponse,
    ConfigVersions,
    FiatTransactionData,
    InAppNotification,
    StreamEvent,
    StreamMessage,
    SupportMessage,
    SupportMessageInput,
    SupportTyping,
);
