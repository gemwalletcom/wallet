package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.HypercoreUSDC
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.firstOrNull
import javax.inject.Inject

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: PerpetualService,
    private val perpetualRepository: PerpetualRepository,
    private val pricesDao: PricesDao,
    private val sessionRepository: SessionRepository,
    private val chains: List<Chain>,
) : SyncPerpetuals {

    override suspend fun syncPerpetuals() {
        chains.forEach { chain ->
            val data = runCatching { perpetualService.getPerpetualsData(chain = chain) }.getOrNull() ?: return@forEach
            perpetualRepository.putPerpetuals(data)
        }
        setupPrices()
    }

    private suspend fun setupPrices() {
        val currency = sessionRepository.getCurrentCurrency()
        val rate = pricesDao.getRates(currency).firstOrNull()?.toDTO() ?: return
        val usdcPrice = AssetPrice(
            assetId = HypercoreUSDC.id,
            price = 1.0,
            priceChangePercentage24h = 0.0,
            updatedAt = 0L,
        )
        pricesDao.insert(usdcPrice.toRecord(rate))
    }
}