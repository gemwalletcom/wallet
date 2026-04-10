package com.gemwallet.android.application.add_asset.coordinators

import com.wallet.core.primitives.AssetId

interface SearchCustomToken {
    suspend operator fun invoke(assetId: AssetId): Boolean
}
