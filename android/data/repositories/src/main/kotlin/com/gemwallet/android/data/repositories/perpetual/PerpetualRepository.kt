package com.gemwallet.android.data.repositories.perpetual

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface PerpetualRepository {
    suspend fun putPerpetuals(items: List<PerpetualData>)

    fun getPerpetuals(query: String? = null): Flow<List<PerpetualData>>

    fun getPerpetual(perpetualId: String): Flow<PerpetualData?>

    fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?>

    suspend fun putPerpetualChartData(data: List<ChartCandleStick>)

    fun getPerpetualChartData(perpetualId: String): Flow<List<ChartCandleStick>>

    suspend fun diffPositions(walletId: WalletId, items: List<PerpetualPosition>)

    fun getPositions(walletId: WalletId): Flow<List<PerpetualPositionData>>

    fun getPositionByPositionId(id: String): Flow<PerpetualPositionData?>

    fun getPositionByPerpetualId(id: String): Flow<PerpetualPositionData?>

    suspend fun putAsset(asset: Asset)

    suspend fun putBalance(walletId: WalletId, assetId: AssetId, balance: PerpetualBalance)

    fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?>

    suspend fun setPinned(perpetualId: String, isPinned: Boolean)
}
