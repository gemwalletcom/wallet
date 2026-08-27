package com.gemwallet.android.model

import com.wallet.core.primitives.PriceAlert
import kotlinx.serialization.Serializable

@Serializable
data class PriceAlertInfo(
    val id: String,
    val priceAlert: PriceAlert,
)