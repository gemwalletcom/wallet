package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualData
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toDB
import com.gemwallet.android.data.service.store.database.entities.toDto
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarketData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemPerpetualStore
import uniffi.gemstone.PerpetualProvider as GemPerpetualProvider

@OptIn(ExperimentalCoroutinesApi::class)
class GemstonePerpetualStore(
    private val perpetualDao: PerpetualDao,
    private val searchDao: SearchDao,
    private val perpetualPositionDao: PerpetualPositionDao,
    private val balancesDao: BalancesDao,
    private val transactionRunner: StoreTransactionRunner,
) : GemPerpetualStore {

    override suspend fun savePerpetuals(data: List<String>) =
        perpetualDao.upsert(data.map { it.decodeJson<PerpetualData>().perpetual.toDB() })

    override suspend fun setPinned(perpetualIds: List<String>, pinned: Boolean) = perpetualDao.setPinned(perpetualIds, pinned)

    override suspend fun deletePerpetuals() = transactionRunner.run {
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

    fun observePerpetuals(query: String? = null): Flow<List<PerpetualData>> {
        val searchQuery = query?.trim().orEmpty()
        if (searchQuery.isEmpty()) {
            return perpetualDao.getPerpetualsData().toPerpetualData()
        }
        return searchDao.hasPerpetualPriorities(searchQuery)
            .map { it > 0 }
            .distinctUntilChanged()
            .flatMapLatest { hasPriority ->
                if (hasPriority) {
                    perpetualDao.searchWithPriority(searchQuery).toPerpetualData()
                } else {
                    perpetualDao.getPerpetualsData().toPerpetualData().map { items -> items.filter { it.matches(searchQuery) } }
                }
            }
    }

    private fun PerpetualData.matches(query: String): Boolean =
        perpetual.name.contains(query, ignoreCase = true) || asset.symbol.contains(query, ignoreCase = true)

    fun observePerpetual(perpetualId: PerpetualId): Flow<PerpetualData?> = perpetualDao.getPerpetual(perpetualId.toIdentifier()).map { it?.toDTO() }

    fun observePerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?> =
        perpetualDao.getPerpetualByAssetId(assetId.toIdentifier()).map { it?.toDTO() }

    fun observePositions(walletId: WalletId): Flow<List<PerpetualPositionData>> =
        perpetualPositionDao.getPositionsData(walletId.id).map { items -> items.mapNotNull { it.toDTO() } }

    fun observePositionByPerpetualId(walletId: WalletId, perpetualId: PerpetualId): Flow<PerpetualPositionData?> =
        perpetualPositionDao.getPositionDataByPerpetual(walletId.id, perpetualId.toIdentifier()).map { it?.toDTO() }

    fun observeBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?> =
        balancesDao.perpetualBalance(walletId.id, assetId.toIdentifier()).map { balance ->
            balance?.let { PerpetualBalance(available = it.available, reserved = it.reserved, withdrawable = it.withdrawable) }
        }

    private fun Flow<List<DbPerpetualData>>.toPerpetualData(): Flow<List<PerpetualData>> = map { items -> items.mapNotNull { it.toDTO() } }
}
