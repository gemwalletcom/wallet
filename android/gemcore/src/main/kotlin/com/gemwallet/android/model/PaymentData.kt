package com.gemwallet.android.model

import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentQuote
import kotlinx.serialization.Serializable

@Serializable
data class PaymentData(
    val quote: PaymentQuote,
    val merchant: PaymentMerchant,
)
