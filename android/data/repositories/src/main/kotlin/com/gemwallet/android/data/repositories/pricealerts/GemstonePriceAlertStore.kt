package com.gemwallet.android.data.repositories.pricealerts

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.ext.id
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

    override suspend fun update(alerts: List<String>, deleteIds: List<String>) {
        val local = priceAlertsDao.getAllPriceAlerts()
        val localIdsByKey = local.associate { it.toDTO().priceAlert.id to it.id }

        val staleIds = deleteIds.mapNotNull { localIdsByKey[it] }
        if (staleIds.isNotEmpty()) {
            priceAlertsDao.delete(staleIds)
        }

        val records = alerts.map { it.decodeJson<PriceAlert>() }
            .map { alert -> alert.toRecord().copy(id = localIdsByKey[alert.id] ?: 0) }
        if (records.isNotEmpty()) {
            priceAlertsDao.put(records)
        }
    }
}
