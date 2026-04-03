package com.gemwallet.android.data.repositories.pricealerts

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.entities.DbPriceAlert
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.id
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
    private val context: Context,
    private val priceAlertsDao: PriceAlertsDao,
    private val configStore: ConfigStore,
) : PriceAlertRepository {

    private val Context.dataStore by preferencesDataStore(name = "price_alerts")

    override suspend fun togglePriceAlerts(enabled: Boolean) {
        context.dataStore.edit { preferences -> preferences[Key.isPriceAlertsEnabled] = enabled }
    }

    override suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean {
        return priceAlertsDao.hasAssetPriceAlerts(assetId.toIdentifier())
    }

    override fun isPriceAlertsEnabled(): Flow<Boolean> = context.dataStore.data
        .map { preferences ->
            if (preferences[Key.isPriceAlertsEnabled] == null) { // TODO: Migration
                togglePriceAlerts(configStore.getBoolean("price_alerts_enabled"))
            }
            preferences[Key.isPriceAlertsEnabled] == true
        }

    override suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo? {
        return priceAlertsDao.getPriceAlert(priceAlertId)?.toDTO()
    }

    override fun getPriceAlerts(assetId: AssetId?): Flow<List<PriceAlertInfo>> {
        return (assetId?.let { priceAlertsDao.getAlerts(it.toIdentifier()) } ?: priceAlertsDao.getAlerts())
            .map { it.toDTO() }
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

    override suspend fun updatePriceAlerts(alerts: List<PriceAlert>) {
        updateSnapshot(
            localAlerts = priceAlertsDao.getAllPriceAlerts(),
            remoteAlerts = alerts,
        )
    }

    override suspend fun updateAssetPriceAlerts(assetId: AssetId, alerts: List<PriceAlert>) {
        val assetIdIdentifier = assetId.toIdentifier()
        updateSnapshot(
            localAlerts = priceAlertsDao.getAllPriceAlerts(assetIdIdentifier),
            remoteAlerts = alerts.filter { it.assetId.toIdentifier() == assetIdIdentifier },
        )
    }

    private suspend fun updateSnapshot(
        localAlerts: List<DbPriceAlert>,
        remoteAlerts: List<PriceAlert>,
    ) {
        val localByKey = localAlerts.associateBy(::identityKey)
        val remoteByKey = remoteAlerts.associateBy(::identityKey)

        val records = remoteByKey.map { (key, alert) ->
            alert.toRecord().copy(id = localByKey[key]?.id ?: 0)
        }
        val staleIds = localByKey.keys
            .subtract(remoteByKey.keys)
            .mapNotNull { key -> localByKey[key]?.id }

        if (staleIds.isNotEmpty()) {
            priceAlertsDao.delete(staleIds)
        }
        if (records.isNotEmpty()) {
            priceAlertsDao.put(records)
        }
    }

    private fun identityKey(priceAlert: PriceAlert): String = priceAlert.id

    private fun identityKey(priceAlert: DbPriceAlert): String = priceAlert.toDTO().priceAlert.id

    private object Key {
        val isPriceAlertsEnabled = booleanPreferencesKey("price_alerts_enabled")
    }
}
