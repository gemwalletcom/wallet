package com.gemwallet.android.application.pricealerts.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection

interface IncludePriceAlert {

    suspend operator fun invoke(
        assetId: AssetId,
        currency: Currency? = null,
        price: Double? = null,
        percentage: Double? = null,
        direction: PriceAlertDirection? = null
    )
}