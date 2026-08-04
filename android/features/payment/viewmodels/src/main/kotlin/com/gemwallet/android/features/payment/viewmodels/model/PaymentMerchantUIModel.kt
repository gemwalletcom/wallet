package com.gemwallet.android.features.payment.viewmodels.model

import uniffi.gemstone.GemPaymentMerchant

data class PaymentMerchantUIModel(
    val name: String,
    val iconUrl: String?,
)

fun GemPaymentMerchant.toUIModel() = PaymentMerchantUIModel(
    name = name,
    iconUrl = iconUrl,
)
