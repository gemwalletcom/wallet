package com.gemwallet.android.domains.perpetual.autoclose

enum class AutocloseError {
    InvalidAmount,
    TriggerMustBeHigher,
    TriggerMustBeLower,
}
