package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PriceAlert
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
}
