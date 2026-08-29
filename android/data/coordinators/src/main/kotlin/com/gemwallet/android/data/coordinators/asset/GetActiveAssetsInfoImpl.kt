package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

class GetActiveAssetsInfoImpl(
    private val getWalletAssets: GetWalletAssets,
) : GetActiveAssetsInfo {
    override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> =
        getWalletAssets()
            .map { items -> items.map { it.toAssetInfoDataAggregate(hideBalance = hideBalance) } }
            .distinctUntilChanged()
}
