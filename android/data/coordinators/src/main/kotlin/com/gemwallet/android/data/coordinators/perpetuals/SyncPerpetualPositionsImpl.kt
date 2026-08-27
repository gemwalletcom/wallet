package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualPositionsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
    private val sessionRepository: SessionRepository,
) : SyncPerpetualPositions {

    override suspend fun syncPerpetualPositions(): PerpetualAccountMode? = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext null
        val address = wallet.hyperliquidAccount?.address ?: return@withContext null
        runCatching { perpetualService.syncPositions(wallet.id.id, Chain.HyperCore.string, address).decodeJson<PerpetualAccountMode>() }.getOrNull()
    }
}
