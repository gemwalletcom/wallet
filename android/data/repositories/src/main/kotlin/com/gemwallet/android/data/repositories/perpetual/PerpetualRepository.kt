package com.gemwallet.android.data.repositories.perpetual

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarketData
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface PerpetualRepository {
    suspend fun putPerpetuals(items: List<PerpetualData>)

    fun getPerpetuals(query: String? = null): Flow<List<PerpetualData>>

    fun getPerpetual(perpetualId: PerpetualId): Flow<PerpetualData?>

    fun getPerpetualByAssetId(assetId: AssetId): Flow<PerpetualData?>






    fun getPositions(walletId: WalletId): Flow<List<PerpetualPositionData>>

    fun getPositionByPerpetualId(walletId: WalletId, id: PerpetualId): Flow<PerpetualPositionData?>



    fun getBalance(walletId: WalletId, assetId: AssetId): Flow<PerpetualBalance?>

    suspend fun setPinned(perpetualId: PerpetualId, isPinned: Boolean)
}
