package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.hyperliquidAccount
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncPerpetualPositionsImpl @Inject constructor(
    private val perpetualService: PerpetualService,
    private val sessionRepository: SessionRepository,
    private val perpetualRepository: PerpetualRepository,
) : SyncPerpetualPositions {

    override suspend fun syncPerpetualPositions(): Unit = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext
        val address = wallet.hyperliquidAccount?.address ?: return@withContext
        val summary = perpetualService.getPositions(Chain.HyperCore, address) ?: return@withContext
        val walletId = wallet.id
        perpetualRepository.putAsset(HypercoreUSDC)
        perpetualRepository.diffPositions(walletId, summary.positions)
        perpetualRepository.putBalance(walletId, HypercoreUSDC.id, summary.balance)
    }
}
