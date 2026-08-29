package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositionByAsset
import com.gemwallet.android.data.adapters.gemstone.GemstonePerpetualStore
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf

@OptIn(ExperimentalCoroutinesApi::class)
class GetPerpetualPositionByAssetImpl(
    private val perpetualStore: GemstonePerpetualStore,
) : GetPerpetualPositionByAsset {

    override fun invoke(walletId: WalletId, assetId: AssetId): Flow<PerpetualPositionData?> =
        perpetualStore.observePerpetualByAssetId(assetId)
            .distinctUntilChanged()
            .flatMapLatest { data ->
                data?.let { perpetualStore.observePositionByPerpetualId(walletId, it.perpetual.id) } ?: flowOf(null)
            }
}
