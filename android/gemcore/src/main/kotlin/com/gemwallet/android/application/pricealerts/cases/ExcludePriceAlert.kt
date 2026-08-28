package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection

interface ExcludePriceAlert {

    suspend operator fun invoke(priceAlert: PriceAlert)

    suspend operator fun invoke(
        assetId: AssetId,
        currency: Currency? = null,
        price: Double? = null,
        percentage: Double? = null,
        direction: PriceAlertDirection? = null,
    )
}
