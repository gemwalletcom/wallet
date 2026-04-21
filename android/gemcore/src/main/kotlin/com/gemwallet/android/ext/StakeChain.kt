package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeChain
import uniffi.gemstone.Config
import uniffi.gemstone.GemStakeChain

fun StakeChain.Companion.isStaked(chain: Chain): Boolean = byChain(chain) != null

fun StakeChain.Companion.byChain(chain: Chain): StakeChain?
    = StakeChain.entries.firstOrNull { it.string == chain.string }

val Chain.canClaimRewards: Boolean
    get() = Config().getStakeConfig(string).canClaimRewards

val Chain.claimAllAvailable: Boolean
    get() = Config().getStakeConfig(string).canClaimAllRewards

val Chain.withdraw: Boolean
    get() = Config().getStakeConfig(string).canWithdraw

val Chain.redelegated: Boolean
    get() = Config().getStakeConfig(string).canRedelegate

val Chain.changeAmountOnUnstake: Boolean
    get() = Config().getStakeConfig(string).changeAmountOnUnstake

fun StakeChain.freezed(): Boolean = when (this) {
    StakeChain.Tron -> true
    StakeChain.Cosmos,
    StakeChain.Injective,
    StakeChain.Sei,
    StakeChain.Celestia,
    StakeChain.Osmosis,
    StakeChain.Solana,
    StakeChain.Sui,
    StakeChain.SmartChain,
    StakeChain.Ethereum,
    StakeChain.Aptos,
    StakeChain.Monad,
    StakeChain.HyperCore -> false
}

fun StakeChain.toGemStakeChain(): GemStakeChain = when (this) {
    StakeChain.Cosmos -> GemStakeChain.COSMOS
    StakeChain.Osmosis -> GemStakeChain.OSMOSIS
    StakeChain.Injective -> GemStakeChain.INJECTIVE
    StakeChain.Sei -> GemStakeChain.SEI
    StakeChain.Celestia -> GemStakeChain.CELESTIA
    StakeChain.Ethereum -> GemStakeChain.ETHEREUM
    StakeChain.Solana -> GemStakeChain.SOLANA
    StakeChain.Sui -> GemStakeChain.SUI
    StakeChain.SmartChain -> GemStakeChain.SMART_CHAIN
    StakeChain.Monad -> GemStakeChain.MONAD
    StakeChain.Tron -> GemStakeChain.TRON
    StakeChain.Aptos -> GemStakeChain.APTOS
    StakeChain.HyperCore -> GemStakeChain.HYPER_CORE
}