package com.gemwallet.android.data.repositories.pricealerts

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class PriceAlertRepositoryImpl(
    private val priceAlertsDao: PriceAlertsDao,
) : PriceAlertRepository {

    override suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean {
        return priceAlertsDao.hasAssetPriceAlerts(assetId.toIdentifier())
    }

    override suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo? {
        return priceAlertsDao.getPriceAlert(priceAlertId)?.toDTO()
    }

    override fun getPriceAlerts(assetId: AssetId?): Flow<List<PriceAlertInfo>> {
        return (assetId?.let { priceAlertsDao.getAlerts(it.toIdentifier()) } ?: priceAlertsDao.getAlerts())
            .map { it.toDTO() }
    }

    override fun getPriceAlertAssetIds(): Flow<List<AssetId>> {
        return priceAlertsDao.getAlerts().map { alerts -> alerts.mapNotNull { it.assetId.toAssetId() } }
    }

    override fun getAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?> {
        return priceAlertsDao.getAssetPriceAlert(assetId.toIdentifier()).mapLatest { it?.toDTO() }
    }

    override suspend fun getSamePriceAlert(priceAlert: PriceAlert): PriceAlertInfo? {
        val samePriceAlert = priceAlertsDao.findSamePriceAlert(
            assetId = priceAlert.assetId.toIdentifier(),
            currency = priceAlert.currency,
            price = priceAlert.price,
            priceDirection = priceAlert.priceDirection,
            pricePercentChange = priceAlert.pricePercentChange
        )
        return samePriceAlert?.toDTO()
    }

    override suspend fun addPriceAlert(priceAlert: PriceAlert) {
        priceAlertsDao.put(listOf(priceAlert.toRecord()))
    }

    override suspend fun disable(priceAlertId: Int) {
        priceAlertsDao.enabled(priceAlertId, false)
    }

    override suspend fun enable(priceAlertId: Int) {
        priceAlertsDao.enabled(priceAlertId, true)
    }

}
