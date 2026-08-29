package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.SetPriceAlertsEnabled
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import uniffi.gemstone.GemPriceAlertService

class IncludePriceAlertImpl(
    private val priceAlertService: GemPriceAlertService,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val setPriceAlertsEnabled: SetPriceAlertsEnabled,
) : IncludePriceAlert {

    override suspend fun invoke(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?
    ) {
        val priceAlert = PriceAlert(
            assetId = assetId,
            currency = currency ?: getCurrentCurrency.getCurrentCurrency(),
            price = price,
            pricePercentChange = percentage,
            priceDirection = direction,
        )
        try {
            priceAlertService.enablePriceAlert(priceAlert.toJson())
            setPriceAlertsEnabled(true)
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }
}
