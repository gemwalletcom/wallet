package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.hyperliquidAccount
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualPositionsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
    private val sessionRepository: SessionRepository,
) : SyncPerpetualPositions {

    override suspend fun syncPerpetualPositions(): Unit = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext
        val address = wallet.hyperliquidAccount?.address ?: return@withContext
        runCatching { perpetualService.syncPositions(wallet.id.id, Chain.HyperCore.string, address) }
    }
}
