package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet

interface EnsureWalletAssets {
    suspend fun ensureWalletAssets(wallet: Wallet, assetIds: List<AssetId>)
}
