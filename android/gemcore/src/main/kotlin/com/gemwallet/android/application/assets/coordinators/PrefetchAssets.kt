package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

interface PrefetchAssets {
    suspend fun prefetchAssets(wallet: Wallet, assetIds: List<AssetId>)
}
