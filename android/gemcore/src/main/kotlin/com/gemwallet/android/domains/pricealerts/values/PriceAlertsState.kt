package com.gemwallet.android.domains.pricealerts.values

import com.wallet.core.primitives.AssetId

sealed class PriceAlertsStateEvent(open val assetId: AssetId? = null) {

    data class Request(override val assetId: AssetId? = null) : PriceAlertsStateEvent(assetId)

    data class Disable(override val assetId: AssetId? = null) : PriceAlertsStateEvent(assetId)

    data class Enable(override val assetId: AssetId? = null) : PriceAlertsStateEvent(assetId)
}
