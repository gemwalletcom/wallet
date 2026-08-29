package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.FeeAssetProvider
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.hasAvailable
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.combine
import uniffi.gemstone.GemAssetConfigService

private val assetConfig = GemAssetConfigService()

@OptIn(ExperimentalCoroutinesApi::class)
class ChainFeeAssetProvider(
    private val chain: Chain,
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
) : FeeAssetProvider {

    private val feeAssetIds = assetConfig.chainFeeAssetIds(chain.string).mapNotNull { it.toAssetId() }.toSet()

    override fun getFeeAssets(): Flow<List<AssetInfo>> = combine(
        getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeAssetsInfoByChain(walletId.id, chain) },
        getCurrentWalletId().flatMapLatest { walletId -> assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain) },
    ) { visible, hidden ->
        (visible + hidden)
            .filter { it.asset.id in feeAssetIds && it.balance.balance.hasAvailable() }
    }
}
