package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemPriceAlertStore

class GemstonePriceAlertStore(
    private val priceAlertsDao: PriceAlertsDao,
) : GemPriceAlertStore {

    override suspend fun getPriceAlerts(assetId: String?): List<String> {
        val records = assetId?.let { priceAlertsDao.getAllPriceAlerts(it) } ?: priceAlertsDao.getAllPriceAlerts()
        return records.map { it.toDTO().priceAlert.toJson() }
    }

    override suspend fun updatePriceAlerts(alerts: List<String>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.decodeJson<PriceAlert>().toRecord() }, deleteIds)
    }

    fun observePriceAlerts(assetId: AssetId?): Flow<List<PriceAlertInfo>> =
        (assetId?.let { priceAlertsDao.getAlerts(it.toIdentifier()) } ?: priceAlertsDao.getAlerts()).map { it.toDTO() }

    fun observeAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?> =
        priceAlertsDao.getAssetPriceAlert(assetId.toIdentifier()).map { it?.toDTO() }

}
