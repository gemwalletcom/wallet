package com.gemwallet.android.domains.pricealerts.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import uniffi.gemstone.GemPriceAlertKind
import uniffi.gemstone.PriceAlertDirection

interface PriceAlertDataAggregate {
    val id: String
    val priceAlert: PriceAlert
    val asset: Asset
    val assetId: AssetId
    val title: String
    val titleBadge: String
    val priceDirection: PriceAlertDirection?
    val price: String
    val percentage: String
    val kind: GemPriceAlertKind
}
