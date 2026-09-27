package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.asset
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain

val Asset.chain: Chain
    get() = id.chain
