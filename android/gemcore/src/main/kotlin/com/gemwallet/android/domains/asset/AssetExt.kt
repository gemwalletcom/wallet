package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.byChain
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain

val Asset.chain: Chain
    get() = id.chain

val Asset.stakeChain: StakeChain?
    get() = StakeChain.byChain(id.chain)
