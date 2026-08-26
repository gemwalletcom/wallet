package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemTransactionsService

@Singleton
class SyncTransactionsImpl @Inject constructor(
    private val transactionsService: GemTransactionsService,
    private val sessionRepository: SessionRepository,
) : SyncTransactions, SyncAssetTransactions {

    override suspend fun syncTransactions(wallet: Wallet) {
        runCatching { transactionsService.sync(wallet.id.id, null) }
    }

    override suspend fun syncAssetTransactions(assetId: AssetId) {
        val wallet = sessionRepository.getCurrentWallet() ?: return
        runCatching { transactionsService.sync(wallet.id.id, assetId.toIdentifier()) }
    }
}
