package com.gemwallet.android.domains.asset

import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.Chain

val AssetData.chain: Chain
    get() = asset.chain
