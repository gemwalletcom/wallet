package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import com.gemwallet.android.domains.gemConfig

fun StakeChain.Companion.byChain(chain: Chain): StakeChain?
    = StakeChain.entries.firstOrNull { it.string == chain.string }

val Chain.changeAmountOnUnstake: Boolean
    get() = gemConfig.getStakeConfig(string).changeAmountOnUnstake

