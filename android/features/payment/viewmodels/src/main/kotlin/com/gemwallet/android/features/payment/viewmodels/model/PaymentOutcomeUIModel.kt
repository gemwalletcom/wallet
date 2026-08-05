package com.gemwallet.android.features.payment.viewmodels.model

import com.wallet.core.primitives.PaymentStatus

enum class PaymentOutcomeUIModel {
    Success,
    Pending,
    Cancelled,
    Expired,
    Failed,
}

fun PaymentStatus.toUIModel() = when (this) {
    PaymentStatus.Succeeded -> PaymentOutcomeUIModel.Success
    PaymentStatus.Processing -> PaymentOutcomeUIModel.Pending
    PaymentStatus.Cancelled -> PaymentOutcomeUIModel.Cancelled
    PaymentStatus.Expired -> PaymentOutcomeUIModel.Expired
    PaymentStatus.Failed,
    PaymentStatus.RequiresAction -> PaymentOutcomeUIModel.Failed
}
