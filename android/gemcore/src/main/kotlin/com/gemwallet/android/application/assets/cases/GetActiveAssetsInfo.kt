package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import kotlinx.coroutines.flow.StateFlow

interface GetActiveAssetsInfo {
    fun assetsInfo(): StateFlow<List<AssetInfoDataAggregate>>
}
