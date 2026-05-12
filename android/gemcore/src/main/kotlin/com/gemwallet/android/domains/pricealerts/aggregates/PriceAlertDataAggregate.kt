package com.gemwallet.android.domains.pricealerts.aggregates

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId

interface PriceAlertDataAggregate {
    val id: Int
    val asset: Asset
    val assetId: AssetId
    val title: String
    val titleBadge: String
    val priceState: ValueDirection
    val price: String
    val percentage: String
    val type: PriceAlertType
    val hasTarget: Boolean
}

enum class PriceAlertType {
    Auto,
    Over,
    Under,
    Increase,
    Decrease,
}
