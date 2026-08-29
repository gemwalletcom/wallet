package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.ExcludePriceAlert
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import uniffi.gemstone.GemPriceAlertService

class ExcludePriceAlertImpl(
    private val priceAlertService: GemPriceAlertService,
    private val getCurrentCurrency: GetCurrentCurrency,
) : ExcludePriceAlert {

    override suspend fun invoke(priceAlert: PriceAlert) {
        try {
            priceAlertService.deletePriceAlerts(listOf(priceAlert.toJson()))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    override suspend fun invoke(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?,
    ) {
        invoke(
            PriceAlert(
                assetId = assetId,
                currency = currency ?: getCurrentCurrency.getCurrentCurrency(),
                price = price,
                pricePercentChange = percentage,
                priceDirection = direction,
            )
        )
    }
}
