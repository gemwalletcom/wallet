package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Chain
import javax.inject.Inject

private val hypercoreUsdcPrice = AssetPrice(
    assetId = HypercoreUSDC.id,
    price = 1.0,
    priceChangePercentage24h = 0.0,
    updatedAt = 0L,
)

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: PerpetualService,
    private val perpetualRepository: PerpetualRepository,
    private val pricesRepository: PricesRepository,
    private val chains: List<Chain>,
) : SyncPerpetuals {

    override suspend fun syncPerpetuals() {
        chains.forEach { chain ->
            val data = runCatching { perpetualService.getPerpetualsData(chain = chain) }.getOrNull() ?: return@forEach
            perpetualRepository.putPerpetuals(data)
        }
        pricesRepository.updatePrice(hypercoreUsdcPrice)
    }
}
