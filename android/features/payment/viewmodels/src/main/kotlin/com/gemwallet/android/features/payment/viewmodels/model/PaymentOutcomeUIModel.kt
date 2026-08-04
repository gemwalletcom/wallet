package com.gemwallet.android.features.payment.viewmodels.model

import uniffi.gemstone.GemPaymentStatus

enum class PaymentOutcomeUIModel {
    Success,
    Pending,
    Cancelled,
    Expired,
    Failed,
}

fun GemPaymentStatus.toUIModel() = when (this) {
    GemPaymentStatus.SUCCEEDED -> PaymentOutcomeUIModel.Success
    GemPaymentStatus.PROCESSING -> PaymentOutcomeUIModel.Pending
    GemPaymentStatus.CANCELLED -> PaymentOutcomeUIModel.Cancelled
    GemPaymentStatus.EXPIRED -> PaymentOutcomeUIModel.Expired
    GemPaymentStatus.FAILED,
    GemPaymentStatus.REQUIRES_ACTION -> PaymentOutcomeUIModel.Failed
}
