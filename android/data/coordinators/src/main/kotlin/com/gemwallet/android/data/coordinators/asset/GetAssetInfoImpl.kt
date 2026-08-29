package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.data.adapters.gemstone.GemstoneAssetStore
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.ExperimentalCoroutinesApi
import com.gemwallet.android.application.session.cases.GetCurrentWalletId

@OptIn(ExperimentalCoroutinesApi::class)
class GetAssetInfoImpl(
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
) : GetAssetInfo {
    override fun invoke(assetId: AssetId): Flow<AssetInfo?> = getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeAssetInfo(walletId.id, assetId) }
}
