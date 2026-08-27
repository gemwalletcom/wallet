package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import javax.inject.Inject
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val chains: List<Chain>,
) : SyncPerpetuals {
    override suspend fun syncPerpetuals() {
        val currency = getCurrentCurrency.getCurrentCurrency().toJson()
        chains.forEach { chain ->
            runCatching { perpetualService.syncMarkets(chain.string, currency) }
        }
    }
}
