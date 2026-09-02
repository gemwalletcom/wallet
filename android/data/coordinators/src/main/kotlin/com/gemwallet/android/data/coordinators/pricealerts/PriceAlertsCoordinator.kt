package com.gemwallet.android.data.coordinators.pricealerts

import android.util.Log
import com.gemwallet.android.application.pricealerts.cases.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.SetAssetPriceAlertEnabled
import com.gemwallet.android.application.pricealerts.cases.SetPriceAlertsEnabled
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import uniffi.gemstone.GemPriceAlertService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.withContext

class PriceAlertsCoordinator(
    private val priceAlertService: GemPriceAlertService,
    private val getCurrentCurrency: GetCurrentCurrency,
) : GetPriceAlertsEnabled, SetPriceAlertsEnabled, IncludePriceAlert, ExcludePriceAlert, SetAssetPriceAlertEnabled {

    private val changes = MutableSharedFlow<Unit>()

    override fun isPriceAlertsEnabled(): Flow<Boolean> = changes
        .onStart { emit(Unit) }
        .map { priceAlertService.isEnabled() }
        .flowOn(Dispatchers.IO)

    override suspend fun setPriceAlertsEnabled(enabled: Boolean) {
        withContext(Dispatchers.IO) { priceAlertService.setEnabled(enabled) }
        changes.emit(Unit)
    }

    override suspend fun setAssetPriceAlertEnabled(assetId: AssetId, enabled: Boolean) {
        if (enabled) {
            includePriceAlert(assetId)
        } else {
            excludePriceAlert(assetId)
        }
    }

    override suspend fun includePriceAlert(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?,
    ) {
        val priceAlert = priceAlert(assetId, currency, price, percentage, direction)
        runCatchingCancellable { priceAlertService.enablePriceAlert(priceAlert.toJson()) }
            .onSuccess { changes.emit(Unit) }
            .onFailure { Log.e(TAG, "enabling the price alert for ${assetId.toIdentifier()} failed", it) }
    }

    override suspend fun excludePriceAlert(priceAlert: PriceAlert) {
        runCatchingCancellable { priceAlertService.deletePriceAlerts(listOf(priceAlert.toJson())) }
            .onFailure { Log.e(TAG, "deleting the price alert for ${priceAlert.assetId.toIdentifier()} failed", it) }
    }

    override suspend fun excludePriceAlert(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?,
    ) = excludePriceAlert(priceAlert(assetId, currency, price, percentage, direction))

    private suspend fun priceAlert(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?,
    ) = PriceAlert(
        assetId = assetId,
        currency = currency ?: getCurrentCurrency.getCurrentCurrency(),
        price = price,
        pricePercentChange = percentage,
        priceDirection = direction,
    )

    private companion object {
        const val TAG = "PriceAlerts"
    }
}
