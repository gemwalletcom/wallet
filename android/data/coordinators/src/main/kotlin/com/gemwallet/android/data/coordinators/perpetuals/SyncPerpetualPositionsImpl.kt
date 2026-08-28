package com.gemwallet.android.data.coordinators.perpetuals

import android.util.Log
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.hyperliquidAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualPositionsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
    private val sessionRepository: SessionRepository,
) : SyncPerpetualPositions {

    override suspend fun syncPerpetualPositions(): PerpetualAccountMode? = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().value?.wallet ?: return@withContext null
        val address = wallet.hyperliquidAccount?.address ?: return@withContext null
        runCatchingCancellable { perpetualService.syncPositions(wallet.id.id, Chain.HyperCore.string, address).decodeJson<PerpetualAccountMode>() }
            .onFailure { Log.e(TAG, "perpetual positions sync failed for ${wallet.id.id}", it) }
            .getOrNull()
    }

    private companion object {
        const val TAG = "SyncPerpetualPositions"
    }
}
