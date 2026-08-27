package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualData
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map

class PerpetualRepositoryImpl(
    private val perpetualDao: PerpetualDao,
    private val perpetualPositionDao: PerpetualPositionDao,
    private val balancesDao: BalancesDao,
    private val searchDao: SearchDao,
    private val perpetualStore: GemstonePerpetualStore,
) : PerpetualRepository {

    override suspend fun putPerpetuals(items: List<PerpetualData>) {
        perpetualStore.putPerpetuals(items)
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    override fun getPerpetuals(query: String?): Flow<List<PerpetualData>> {
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
                    perpetualDao.getPerpetualsData().toPerpetualData()
                        .map { items -> items.filter { it.matches(searchQuery) } }
                }
            }
    }

    private fun Flow<List<DbPerpetualData>>.toPerpetualData(): Flow<List<PerpetualData>> =
        map { items -> items.mapNotNull { it.toDTO() } }

    private fun PerpetualData.matches(query: String): Boolean =
        perpetual.name.contains(query, ignoreCase = true) ||
            asset.symbol.contains(query, ignoreCase = true)

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?> {
        return perpetualDao.getPerpetual(perpetualId.toIdentifier()).map { it?.toDTO() }
    }

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?> {
        return perpetualDao.getPerpetualByAssetId(assetId.toIdentifier()).map { it?.toDTO() }
    }

    override fun getPositions(walletId: WalletId): Flow<List<PerpetualPositionData>> {
        return perpetualPositionDao.getPositionsData(walletId.id).map { items -> items.mapNotNull { it.toDTO() } }
    }

    override fun getPositionByPerpetualId(walletId: WalletId, id: PerpetualId): Flow<PerpetualPositionData?> {
        return perpetualPositionDao.getPositionDataByPerpetual(walletId.id, id.toIdentifier()).map { it?.toDTO() }
    }

    override fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?> {
        return balancesDao.perpetualBalance(walletId.id, assetId.toIdentifier())
            .map { it?.let { PerpetualBalance(available = it.available, reserved = it.reserved, withdrawable = it.withdrawable) } }
    }

}
