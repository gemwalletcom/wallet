package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.entities.toDB
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toDto
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class PerpetualRepositoryImpl(
    private val perpetualDao: PerpetualDao,
    private val perpetualPositionDao: PerpetualPositionDao,
    private val assetsDao: AssetsDao,
    private val balancesDao: BalancesDao,
) : PerpetualRepository {

    override suspend fun putPerpetuals(items: List<PerpetualData>) {
        assetsDao.insert(items.map { it.asset.toRecord() })
        perpetualDao.upsert(items.map { it.perpetual.toDB() })
    }

    override fun getPerpetuals(query: String?): Flow<List<PerpetualData>> {
        val needle = query?.trim().orEmpty()
        return perpetualDao.getPerpetualsData().map { items ->
            items.mapNotNull { it.toDTO() }.filter { needle.isEmpty() || it.matches(needle) }
        }
    }

    private fun PerpetualData.matches(needle: String): Boolean =
        perpetual.name.contains(needle, ignoreCase = true) ||
            asset.symbol.contains(needle, ignoreCase = true) ||
            asset.name.contains(needle, ignoreCase = true)

    override fun getPerpetual(perpetualId: String): Flow<PerpetualData?> {
        return perpetualDao.getPerpetual(perpetualId).map { it?.toDTO() }
    }

    override suspend fun putPerpetualChartData(data: List<ChartCandleStick>) {
        TODO("Not yet implemented")
    }

    override fun getPerpetualChartData(perpetualId: String): Flow<List<ChartCandleStick>> {
        TODO("Not yet implemented")
    }

    override suspend fun diffPositions(walletId: WalletId, items: List<PerpetualPosition>) {
        perpetualPositionDao.diffPositions(walletId.id, items.map { it.toDB(walletId.id) })
    }

    override fun getPositions(walletId: WalletId): Flow<List<PerpetualPositionData>> {
        return perpetualPositionDao.getPositionsData(walletId.id).map { items -> items.mapNotNull { it.toDTO() } }
    }

    override fun getPositionByPositionId(id: String): Flow<PerpetualPositionData?> {
        return perpetualPositionDao.getPositionData(id).map { it?.toDTO() }
    }

    override fun getPositionByPerpetualId(id: String): Flow<PerpetualPositionData?> {
        return perpetualPositionDao.getPositionDataByPerpetual(id).map { it?.toDTO() }
    }

    override suspend fun putAsset(asset: Asset) {
        assetsDao.insert(asset.toRecord())
    }

    override suspend fun putBalance(walletId: WalletId, assetId: AssetId, balance: PerpetualBalance) {
        balancesDao.insert(
            DbBalance(
                assetId = assetId.toIdentifier(),
                walletId = walletId.id,
                available = balance.available.toString(),
                availableAmount = balance.available,
                reserved = balance.reserved.toString(),
                reservedAmount = balance.reserved,
                withdrawable = balance.withdrawable.toString(),
                withdrawableAmount = balance.withdrawable,
                totalAmount = balance.available + balance.reserved,
                isActive = true,
                updatedAt = System.currentTimeMillis(),
            )
        )
    }

    override fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?> {
        return balancesDao.perpetualBalance(walletId.id, assetId.toIdentifier())
            .map { it?.let { PerpetualBalance(available = it.available, reserved = it.reserved, withdrawable = it.withdrawable) } }
    }

    override suspend fun setPinned(perpetualId: String, isPinned: Boolean) {
        perpetualDao.setPinned(perpetualId, isPinned)
    }
}
