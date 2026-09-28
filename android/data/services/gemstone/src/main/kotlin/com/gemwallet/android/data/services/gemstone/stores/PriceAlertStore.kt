package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.PriceAlertsDao
import com.gemwallet.android.data.services.store.database.entities.toPriceAlert
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemPriceAlertStore
import uniffi.gemstone.PriceAlertFormatter

class GemstonePriceAlertStore(private val priceAlertsDao: PriceAlertsDao, private val priceAlertFormatter: PriceAlertFormatter) : GemPriceAlertStore {

    override suspend fun getPriceAlerts(assetId: String?): List<uniffi.gemstone.PriceAlert> {
        val records = assetId?.let { priceAlertsDao.getAllPriceAlerts(it) } ?: priceAlertsDao.getAllPriceAlerts()
        return records.map { it.toPriceAlert().toGem() }
    }

    override suspend fun updatePriceAlerts(alerts: List<uniffi.gemstone.PriceAlert>, deleteIds: List<String>) {
        priceAlertsDao.update(alerts.map { it.toPrimitives().toRecord(priceAlertFormatter.alertId(it)) }, deleteIds)
    }
}
