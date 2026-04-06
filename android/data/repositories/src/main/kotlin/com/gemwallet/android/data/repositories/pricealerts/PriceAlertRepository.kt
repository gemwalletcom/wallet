package com.gemwallet.android.data.repositories.pricealerts

import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.flow.Flow

interface PriceAlertRepository {
    fun isPriceAlertsEnabled(): Flow<Boolean>

    suspend fun togglePriceAlerts(enabled: Boolean)

    suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean

    suspend fun getSamePriceAlert(priceAlert: PriceAlert): PriceAlertInfo?

    fun getPriceAlerts(assetId: AssetId? = null): Flow<List<PriceAlertInfo>>

    fun getAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?>

    suspend fun addPriceAlert(priceAlert: PriceAlert)

    suspend fun updatePriceAlerts(alerts: List<PriceAlert>)

    suspend fun updateAssetPriceAlerts(assetId: AssetId, alerts: List<PriceAlert>)

    suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo?

    suspend fun disable(priceAlertId: Int)

    suspend fun enable(priceAlertId: Int)
}
