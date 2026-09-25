package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn

@OptIn(ExperimentalCoroutinesApi::class)
class WalletAssetsCoordinator(assetsQuery: AssetsQuery, getCurrentWalletId: GetCurrentWalletId, scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) : GetWalletAssets {

    private val walletAssets: StateFlow<List<AssetInfo>> = getCurrentWalletId()
        .flatMapLatest { walletId -> assetsQuery(walletId) }
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun invoke(): StateFlow<List<AssetInfo>> = walletAssets
}
