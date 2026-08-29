package com.gemwallet.android.data.coordinators.asset_select

import com.gemwallet.android.application.asset_select.cases.GetChainAssets
import com.gemwallet.android.application.assets.cases.SyncBalances
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.first

@OptIn(ExperimentalCoroutinesApi::class)
class GetChainAssetsImpl(
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
    private val syncBalances: SyncBalances,
) : GetChainAssets {
    override fun invoke(chain: Chain): Flow<List<AssetInfo>> = getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeAssetsInfoByChain(walletId.id, chain) }

    override fun hidden(chain: Chain): Flow<List<AssetInfo>> = getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain) }

    override suspend fun updateBalances(chain: Chain) {
        val assets = getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeAssetsInfoByChain(walletId.id, chain) }.first() +
            getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain) }.first()
        syncBalances(assets)
    }
}
