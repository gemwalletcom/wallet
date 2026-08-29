package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.shareIn

@OptIn(ExperimentalCoroutinesApi::class)
class WalletAssetsCoordinator(
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletAssets {

    private val walletAssets: Flow<List<AssetInfo>> = getCurrentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfo(walletId.id) }
        .shareIn(scope, SharingStarted.Eagerly, replay = 1)

    override fun invoke(): Flow<List<AssetInfo>> = walletAssets

    override fun invoke(walletId: WalletId): Flow<List<AssetInfo>> = assetStore.observeAssetsInfo(walletId.id).flowOn(Dispatchers.IO)

    override fun invoke(assetIds: List<AssetId>): Flow<List<AssetInfo>> = byIdentifiers(assetIds.map { it.toIdentifier() })

    override fun byIdentifiers(assetIds: List<String>): Flow<List<AssetInfo>> = getCurrentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfo(walletId.id, assetIds) }
        .flowOn(Dispatchers.IO)
}
