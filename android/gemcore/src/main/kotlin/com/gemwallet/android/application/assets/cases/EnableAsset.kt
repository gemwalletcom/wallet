package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId

interface EnableAsset {
    suspend operator fun invoke(walletId: WalletId, assetId: AssetId, enabled: Boolean = true)

    suspend operator fun invoke(walletId: WalletId, assetIds: List<AssetId>, enabled: Boolean = true)
}
