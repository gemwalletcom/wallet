package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.assetConfig
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAsset
import com.wallet.core.primitives.ChainType
import uniffi.gemstone.Config


private val chainAssetCache: Map<Chain, ChainAsset> by lazy {
    Chain.entries.associateWith { chain ->
        assetConfig.chainAsset(chain.string).decodeJson()
    }
}

private fun Chain.chainAsset(): ChainAsset {
    return chainAssetCache[this] ?: throw IllegalArgumentException("Unsupported chain: $string")
}

fun Chain.assetType(): AssetType? = Config().getChainConfig(string).defaultAssetType?.toPrimitives()

fun Chain.isStakeSupported(): Boolean = Config().getChainConfig(this.string).isStakeSupported

fun Chain.asset(): Asset {
    return chainAsset().asset
}

fun Chain.networkName(): String {
    return chainAsset().networkName
}

fun Chain.Companion.available() = Chain.entries.toSet()


fun Chain.toChainType(): ChainType = Config().getChainConfig(string).chainType.toPrimitives()



fun Chain.isSwapSupport(): Boolean = Config().getChainConfig(string).isSwapSupported

fun Chain.isMemoSupport() = Config().getChainConfig(string).isMemoSupported

fun uniffi.gemstone.Chain.requireChain(): Chain = requireNotNull(Chain.entries.firstOrNull { it.string == this }) { "unknown chain: $this" }
