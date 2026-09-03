package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import uniffi.gemstone.Config

fun StakeChain.Companion.byChain(chain: Chain): StakeChain?
    = StakeChain.entries.firstOrNull { it.string == chain.string }

val Chain.withdraw: Boolean
    get() = Config().getStakeConfig(string).canWithdraw

val Chain.changeAmountOnUnstake: Boolean
    get() = Config().getStakeConfig(string).changeAmountOnUnstake

