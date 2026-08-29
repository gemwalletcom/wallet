package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection

interface ExcludePriceAlert {
    suspend fun excludePriceAlert(priceAlert: PriceAlert)

    suspend fun excludePriceAlert(
        assetId: AssetId,
        currency: Currency? = null,
        price: Double? = null,
        percentage: Double? = null,
        direction: PriceAlertDirection? = null,
    )
}
