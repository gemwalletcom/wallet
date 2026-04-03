package com.gemwallet.android.services

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
) {

    suspend fun sync() = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().firstOrNull()?.wallet
        buildList {
            add(async { buyRepository.sync() })
            wallet?.let { wallet ->
                add(async { syncTransactions.syncTransactions(wallet) })
            }
        }
            .awaitAll()
        syncSubscription.syncSubscription(walletsRepository.getAll().firstOrNull() ?: emptyList())
    }
}
