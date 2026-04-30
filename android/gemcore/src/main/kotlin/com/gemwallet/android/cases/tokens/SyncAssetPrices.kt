package com.gemwallet.android.cases.tokens

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency

interface SyncAssetPrices {
    suspend operator fun invoke(assetIds: List<AssetId>, currency: Currency)
}
