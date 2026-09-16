package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

interface GetWalletAssets {
    operator fun invoke(): StateFlow<List<AssetInfo>>

    operator fun invoke(walletId: WalletId): Flow<List<AssetInfo>>

    operator fun invoke(assetIds: List<AssetId>): Flow<List<AssetInfo>>

    fun byIdentifiers(assetIds: List<String>): Flow<List<AssetInfo>>
}
