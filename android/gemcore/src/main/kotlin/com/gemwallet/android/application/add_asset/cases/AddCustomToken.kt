package com.gemwallet.android.application.add_asset.cases

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

interface AddCustomToken {
    suspend operator fun invoke(chain: Chain, assetId: AssetId)
}
