package com.gemwallet.android.data.repositories.pricealerts

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
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
}
