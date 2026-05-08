package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

interface EnableAsset {
    suspend operator fun invoke(walletId: WalletId, assetId: AssetId)

    suspend operator fun invoke(walletId: WalletId, assetIds: List<AssetId>)
}
