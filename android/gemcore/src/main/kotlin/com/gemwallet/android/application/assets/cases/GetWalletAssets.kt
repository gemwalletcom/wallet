package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetData
import kotlinx.coroutines.flow.StateFlow

interface GetWalletAssets {
    operator fun invoke(): StateFlow<List<AssetData>>
}
