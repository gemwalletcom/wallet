package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive

class ExcludePriceAlertImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val sessionRepository: SessionRepository,
    private val priceAlertRepository: PriceAlertRepository,
) : ExcludePriceAlert {
    override suspend fun invoke(priceAlertId: Int) {
        priceAlertRepository.getPriceAlert(priceAlertId)?.priceAlert?.let { priceAlert ->
            invoke(
                priceAlert.assetId,
                Currency.entries.firstOrNull { it.string == priceAlert.currency },
                priceAlert.price,
                priceAlert.pricePercentChange,
                priceAlert.priceDirection
            )
        }
    }

    override suspend fun invoke(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?,
    ) {
        val currency = currency?.string ?: sessionRepository.getCurrentCurrency().string
        val priceAlert = PriceAlert(
            assetId = assetId,
            currency = currency,
            price = price,
            pricePercentChange = percentage,
            priceDirection = direction,
        )
        val priceAlertInfo = priceAlertRepository.getSamePriceAlert(priceAlert) ?: return
        priceAlertRepository.disable(priceAlertInfo.id)
        try {
            gemDeviceApiClient.excludePriceAlert(listOf(priceAlertInfo.priceAlert))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }
}
