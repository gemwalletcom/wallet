package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import uniffi.gemstone.Config

fun StakeChain.Companion.isStaked(chain: Chain): Boolean = byChain(chain) != null

fun StakeChain.Companion.byChain(chain: Chain): StakeChain?
    = StakeChain.entries.firstOrNull { it.string == chain.string }

val Chain.canClaimRewards: Boolean
    get() = Config().getStakeConfig(string).canClaimRewards

val Chain.claimAllAvailable: Boolean
    get() = Config().getStakeConfig(string).canClaimAllRewards

val Chain.withdraw: Boolean
    get() = Config().getStakeConfig(string).canWithdraw

val Chain.changeAmountOnUnstake: Boolean
    get() = Config().getStakeConfig(string).changeAmountOnUnstake

fun StakeChain.freezed(): Boolean = Config().getStakeConfig(string).usesFreeze
