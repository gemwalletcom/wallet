package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.cases.ObserveToken
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.ExperimentalCoroutinesApi
import com.gemwallet.android.application.session.cases.GetCurrentWalletId

@OptIn(ExperimentalCoroutinesApi::class)
class ObserveTokenImpl(
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
) : ObserveToken {

    override fun invoke(assetId: AssetId): Flow<Asset?> {
        return getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeTokenInfo(walletId.id, assetId).map { it?.asset } }
    }
}
