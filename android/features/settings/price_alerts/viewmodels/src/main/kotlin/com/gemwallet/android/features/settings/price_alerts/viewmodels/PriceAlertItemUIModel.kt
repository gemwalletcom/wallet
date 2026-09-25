package com.gemwallet.android.features.settings.price_alerts.viewmodels

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PriceAlert
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPriceAlertItem

class PriceAlertItemUIModel(item: GemPriceAlertItem) {
    val id: String = item.id
    val asset: Asset = item.data.asset.toPrimitives()
    val priceAlert: PriceAlert = item.data.priceAlert.toPrimitives()
    val row: GemAssetItemRow = item.row.row
}
