package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

fun mockAssetId(
    chain: Chain = Chain.Bitcoin,
    tokenId: String? = null,
) = AssetId(
    chain = chain,
    tokenId = tokenId,
)
