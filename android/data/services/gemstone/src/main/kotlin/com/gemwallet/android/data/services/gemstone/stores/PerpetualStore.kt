package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.BalancesDao
import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.PerpetualPositionDao
import com.gemwallet.android.data.services.store.database.StoreTransactionRunner
import com.gemwallet.android.data.services.store.database.entities.toDB
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toDto
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemPerpetualStore
import uniffi.gemstone.PerpetualProvider as GemPerpetualProvider

class GemstonePerpetualStore(private val perpetualDao: PerpetualDao, private val perpetualPositionDao: PerpetualPositionDao, private val balancesDao: BalancesDao, private val transactionRunner: StoreTransactionRunner) : GemPerpetualStore {

    override suspend fun getPerpetuals(names: List<String>): List<uniffi.gemstone.Perpetual> = perpetualDao.getPerpetualsByNames(names).map { it.toDTO().toGem() }

    override suspend fun savePerpetuals(data: List<uniffi.gemstone.PerpetualData>) = perpetualDao.upsert(data.map { it.perpetual.toPrimitives().toDB() })

    override suspend fun setPinned(perpetualIds: List<String>, pinned: Boolean) = perpetualDao.setPinned(perpetualIds, pinned)

    override suspend fun clearPerpetuals(collateralAssetIds: List<String>) = transactionRunner.run {
        perpetualPositionDao.deleteAll()
        perpetualDao.deleteAll()
        collateralAssetIds.forEach { balancesDao.deleteByAssetId(it) }
    }

    override suspend fun getPositions(walletId: String, provider: GemPerpetualProvider): List<uniffi.gemstone.PerpetualPosition> = perpetualPositionDao.getPositionsByProvider(walletId, provider.toPrimitives()).map { it.toDto().toGem() }

    override suspend fun updateMarket(market: uniffi.gemstone.PerpetualMarketData) {
        perpetualDao.updateMarket(
            coin = market.coin,
            price = market.price,
            pricePercentChange24h = market.pricePercentChange24h,
            openInterest = market.openInterest,
            volume24h = market.volume24h,
            funding = market.funding,
        )
    }

    override suspend fun updatePrices(prices: Map<String, Double>) = perpetualDao.updatePrices(prices)

    override suspend fun getPositionIds(walletId: String, provider: GemPerpetualProvider): List<String> = perpetualPositionDao.getPositionsByProvider(walletId, provider.toPrimitives()).map { it.id }

    override suspend fun updatePositions(walletId: String, positions: List<uniffi.gemstone.PerpetualPosition>, deleteIds: List<String>) = putPositions(WalletId(walletId), positions.map { it.toPrimitives() }, deleteIds)

    private suspend fun putPositions(walletId: WalletId, positions: List<PerpetualPosition>, deleteIds: List<String>) {
        if (deleteIds.isEmpty() && positions.isEmpty()) return
        perpetualPositionDao.deleteAndUpsert(walletId.id, deleteIds, positions.map { it.toDB(walletId.id) })
    }
}
