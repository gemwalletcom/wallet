package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.wallet.core.primitives.Chain
import javax.inject.Inject

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: PerpetualService,
    private val perpetualRepository: PerpetualRepository,
    private val chains: List<Chain>,
) : SyncPerpetuals {

    override suspend fun syncPerpetuals() {
        chains.forEach { chain ->
            val data = runCatching { perpetualService.getPerpetualsData(chain = chain) }.getOrNull() ?: return@forEach
            perpetualRepository.putPerpetuals(data)
        }
    }
}