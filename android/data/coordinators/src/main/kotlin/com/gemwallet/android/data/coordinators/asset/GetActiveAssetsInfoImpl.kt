package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import uniffi.gemstone.GemAssetRow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.stateIn

class GetActiveAssetsInfoImpl(
    getWalletAssets: GetWalletAssets,
    userConfig: UserConfig,
    row: GemAssetRow,
    scope: CoroutineScope = CoroutineScope(Dispatchers.Default),
) : GetActiveAssetsInfo {

    private val assetsInfo: StateFlow<List<AssetInfoDataAggregate>> =
        combine(getWalletAssets(), userConfig.isHideBalances()) { items, hideBalance ->
            items.toAssetInfoDataAggregates(naming = row.title, hideBalance = hideBalance)
        }
        .distinctUntilChanged()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>> = assetsInfo
}
