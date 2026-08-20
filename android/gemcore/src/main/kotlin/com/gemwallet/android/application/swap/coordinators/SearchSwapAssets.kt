package com.gemwallet.android.application.swap.coordinators

import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

interface SearchSwapAssets {
    operator fun invoke(
        wallet: Wallet?,
        query: String,
        swapItemType: SwapItemType,
        oppositeAssetId: AssetId?,
    ): Flow<List<AssetInfo>>
}
