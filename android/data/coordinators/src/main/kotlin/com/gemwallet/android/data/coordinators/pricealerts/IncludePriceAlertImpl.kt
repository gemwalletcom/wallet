package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.SetPriceAlertsEnabled
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import uniffi.gemstone.GemPriceAlertService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson

class IncludePriceAlertImpl(
    private val priceAlertService: GemPriceAlertService,
    private val sessionRepository: SessionRepository,
    private val priceAlertRepository: PriceAlertRepository,
    private val setPriceAlertsEnabled: SetPriceAlertsEnabled,
) : IncludePriceAlert {

    override suspend fun invoke(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?
    ) {
        val currency = currency ?: sessionRepository.getCurrentCurrency()
        val priceAlert = PriceAlert(
            assetId = assetId,
            currency = currency,
            price = price,
            pricePercentChange = percentage,
            priceDirection = direction,
        )
        priceAlertRepository.getSamePriceAlert(priceAlert)?.let {
            priceAlertRepository.enable(it.id)
        } ?: priceAlertRepository.addPriceAlert(priceAlert)
        enablePriceAlertsIfNeeded()

        try {
            priceAlertService.addPriceAlerts(alerts = listOf(priceAlert.toJson()))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private suspend fun enablePriceAlertsIfNeeded() {
        try {
            setPriceAlertsEnabled(true)
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }
}
