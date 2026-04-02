package com.gemwallet.android.services

import com.gemwallet.android.BuildConfig
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.application.pricealerts.coordinators.SyncPriceAlerts
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncService @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val syncTransactions: SyncTransactions,
    private val buyRepository: BuyRepository,
    private val syncSubscription: SyncSubscription,
    private val syncPerpetuals: SyncPerpetuals,
    private val syncPriceAlerts: SyncPriceAlerts,
) {

    suspend fun sync() = withContext(Dispatchers.IO) {
        listOf(
            async { syncTransactions.syncTransactions(sessionRepository.session().firstOrNull()?.wallet ?: return@async) },
            async { buyRepository.sync() },
            async { syncPriceAlerts.syncPriceAlerts() },
        ).awaitAll()
        syncSubscription.syncSubscription(walletsRepository.getAll().firstOrNull() ?: emptyList())

        if (BuildConfig.DEBUG) {
            syncPerpetuals.syncPerpetuals()
        }
    }
}