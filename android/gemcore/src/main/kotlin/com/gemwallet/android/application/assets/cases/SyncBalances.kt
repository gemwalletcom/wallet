package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo

interface SyncBalances {
    suspend operator fun invoke(assets: List<AssetInfo>)
}
