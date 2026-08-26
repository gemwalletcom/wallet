package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.wallet.core.primitives.Chain
import javax.inject.Inject
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
    private val chains: List<Chain>,
) : SyncPerpetuals {

    override suspend fun syncPerpetuals() {
        chains.forEach { chain ->
            runCatching { perpetualService.syncMarkets(chain.string) }
        }
    }
}
