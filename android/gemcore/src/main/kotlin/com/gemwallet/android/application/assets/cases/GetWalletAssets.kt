package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.flow.StateFlow

interface GetWalletAssets {
    operator fun invoke(): StateFlow<List<AssetInfo>>
}
