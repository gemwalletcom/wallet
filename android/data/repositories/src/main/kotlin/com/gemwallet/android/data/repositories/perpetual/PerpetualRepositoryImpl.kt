package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import com.gemwallet.android.data.service.store.database.SearchDao
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
    private val perpetualStore: GemstonePerpetualStore,
    private val searchDao: SearchDao,
) : PerpetualRepository {

    @OptIn(ExperimentalCoroutinesApi::class)
    override fun getPerpetuals(query: String?): Flow<List<PerpetualData>> {
        val searchQuery = query?.trim().orEmpty()
        if (searchQuery.isEmpty()) {
            return perpetualStore.observePerpetuals()
        }
        return searchDao.hasPerpetualPriorities(searchQuery)
            .map { it > 0 }
            .distinctUntilChanged()
            .flatMapLatest { hasPriority ->
                if (hasPriority) {
                    perpetualStore.observePerpetualsWithPriority(searchQuery)
                } else {
                    perpetualStore.observePerpetuals().map { items -> items.filter { it.matches(searchQuery) } }
                }
            }
    }

    private fun PerpetualData.matches(query: String): Boolean =
        perpetual.name.contains(query, ignoreCase = true) ||
            asset.symbol.contains(query, ignoreCase = true)

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?> = perpetualStore.observePerpetual(perpetualId)

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?> = perpetualStore.observePerpetualByAssetId(assetId)

    override fun getPositions(walletId: WalletId): Flow<List<PerpetualPositionData>> = perpetualStore.observePositions(walletId)

    override fun getPositionByPerpetualId(walletId: WalletId, id: PerpetualId): Flow<PerpetualPositionData?> =
        perpetualStore.observePositionByPerpetualId(walletId, id)

    override fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?> = perpetualStore.observeBalance(walletId, assetId)

}
