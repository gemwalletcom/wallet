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
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
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

    override fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?> {
        return perpetualDao.getPerpetual(perpetualId.toIdentifier()).map { it?.toDTO() }
    }

    override fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?> {
        return perpetualDao.getPerpetualByAssetId(assetId.toIdentifier()).map { it?.toDTO() }
    }

    override suspend fun putPerpetualChartData(data: List<ChartCandleStick>) {
        TODO("Not yet implemented")
    }

    override fun getPerpetualChartData(perpetualId: PerpetualId): Flow<List<ChartCandleStick>> {
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

    override fun getPositionByPerpetualId(id: PerpetualId): Flow<PerpetualPositionData?> {
        return perpetualPositionDao.getPositionDataByPerpetual(id.toIdentifier()).map { it?.toDTO() }
    }

    override suspend fun putAsset(asset: Asset) {
        assetsDao.insert(asset.toRecord())
    }

    override suspend fun putBalance(walletId: WalletId, asset: Asset, balance: PerpetualBalance) {
        balancesDao.insert(balance.toDbBalance(walletId, asset, System.currentTimeMillis()))
    }

    override fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?> {
        return balancesDao.perpetualBalance(walletId.id, assetId.toIdentifier())
            .map { it?.let { PerpetualBalance(available = it.available, reserved = it.reserved, withdrawable = it.withdrawable) } }
    }

    override suspend fun setPinned(perpetualId: PerpetualId, isPinned: Boolean) {
        perpetualDao.setPinned(perpetualId.toIdentifier(), isPinned)
    }
}

internal fun PerpetualBalance.toDbBalance(
    walletId: WalletId,
    asset: Asset,
    updatedAt: Long,
): DbBalance = DbBalance(
    assetId = asset.id.toIdentifier(),
    walletId = walletId.id,
    available = available.toAtomicUnits(asset.decimals),
    availableAmount = available,
    reserved = reserved.toAtomicUnits(asset.decimals),
    reservedAmount = reserved,
    withdrawable = withdrawable.toAtomicUnits(asset.decimals),
    withdrawableAmount = withdrawable,
    totalAmount = available + reserved,
    isActive = true,
    updatedAt = updatedAt,
)

private fun Double.toAtomicUnits(decimals: Int): String =
    Crypto(toBigDecimal(), decimals).atomicValue.toString()
