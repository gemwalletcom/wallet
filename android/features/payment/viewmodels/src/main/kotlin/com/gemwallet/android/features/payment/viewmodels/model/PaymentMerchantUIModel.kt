package com.gemwallet.android.features.payment.viewmodels.model

import com.wallet.core.primitives.PaymentMerchant

data class PaymentMerchantUIModel(
    val name: String,
    val iconUrl: String?,
)

fun PaymentMerchant.toUIModel() = PaymentMerchantUIModel(
    name = name,
    iconUrl = iconUrl,
)
