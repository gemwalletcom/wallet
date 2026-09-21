package com.gemwallet.android.domains.pricealerts.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import uniffi.gemstone.GemPriceAlertText
import uniffi.gemstone.PriceAlertDirection

interface PriceAlertDataAggregate {
    val id: String
    val priceAlert: PriceAlert
    val asset: Asset
    val assetId: AssetId
    val rankScore: Int
    val title: String
    val titleBadge: String?
    val priceDirection: PriceAlertDirection?
    val prefix: GemPriceAlertText
    val suffix: GemPriceAlertText
}
