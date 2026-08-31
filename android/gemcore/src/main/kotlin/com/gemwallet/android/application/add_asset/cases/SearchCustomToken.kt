package com.gemwallet.android.application.add_asset.cases

import com.wallet.core.primitives.AssetId

interface SearchCustomToken {
    suspend operator fun invoke(assetId: AssetId): Boolean
}
