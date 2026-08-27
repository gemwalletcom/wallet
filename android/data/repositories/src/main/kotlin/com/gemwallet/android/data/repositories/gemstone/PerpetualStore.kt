package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toDB
import com.gemwallet.android.data.service.store.database.entities.toDto
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualMarketData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemPerpetualStore
import uniffi.gemstone.PerpetualProvider as GemPerpetualProvider

class GemstonePerpetualStore(
    private val perpetualDao: PerpetualDao,
    private val perpetualPositionDao: PerpetualPositionDao,
    private val assetsDao: AssetsDao,
    private val balancesDao: BalancesDao,
) : GemPerpetualStore {

    override suspend fun savePerpetuals(data: List<String>) =
        putPerpetuals(data.map { it.decodeJson<PerpetualData>() })

    private suspend fun putPerpetuals(items: List<PerpetualData>) {
        assetsDao.insert(items.map { it.asset.toRecord() })
        perpetualDao.upsert(items.map { it.perpetual.toDB() })
    }

    override suspend fun setPinned(perpetualIds: List<String>, pinned: Boolean) {
        perpetualIds.forEach { perpetualDao.setPinned(it, pinned) }
    }

    override suspend fun clear() {
        perpetualPositionDao.deleteAll()
        perpetualDao.deleteAll()
        balancesDao.deleteByAssetId(HypercoreUSDC.id.toIdentifier())
    }

    override suspend fun getPositions(walletId: String, provider: GemPerpetualProvider): List<String> =
        perpetualPositionDao.getPositionsByProvider(walletId, provider.toPrimitives()).map { it.toDto().toJson() }

    override suspend fun updateMarket(market: String) {
        val data = market.decodeJson<PerpetualMarketData>()
        perpetualDao.updateMarket(
            coin = data.coin,
            price = data.price,
            pricePercentChange24h = data.pricePercentChange24h,
            openInterest = data.openInterest,
            volume24h = data.volume24h,
            funding = data.funding,
        )
    }

    override suspend fun updatePrices(prices: Map<String, Double>) = perpetualDao.updatePrices(prices)

    override suspend fun getPositionIds(walletId: String, provider: GemPerpetualProvider): List<String> =
        perpetualPositionDao.getPositionsByProvider(walletId, provider.toPrimitives()).map { it.id }

    override suspend fun updatePositions(walletId: String, positions: List<String>, deleteIds: List<String>) =
        putPositions(WalletId(walletId), positions.map { it.decodeJson<PerpetualPosition>() }, deleteIds)

    private suspend fun putPositions(walletId: WalletId, positions: List<PerpetualPosition>, deleteIds: List<String>) {
        if (deleteIds.isEmpty() && positions.isEmpty()) return
        perpetualPositionDao.applyDiff(walletId.id, deleteIds, positions.map { it.toDB(walletId.id) })
    }

    private fun GemPerpetualProvider.toPrimitives(): PerpetualProvider = when (this) {
        GemPerpetualProvider.HYPERCORE -> PerpetualProvider.Hypercore
    }
}
