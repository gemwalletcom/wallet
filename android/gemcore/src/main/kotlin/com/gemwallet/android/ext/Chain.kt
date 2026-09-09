package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.assetConfig
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAsset
import com.gemwallet.android.domains.gemConfig
import uniffi.gemstone.ChainConfig


private val chainAssetCache: Map<Chain, ChainAsset> by lazy {
    Chain.entries.associateWith { chain ->
        assetConfig.chainAsset(chain.string).toPrimitives()
    }
}

private fun Chain.chainAsset(): ChainAsset {
    return chainAssetCache[this] ?: throw IllegalArgumentException("Unsupported chain: $string")
}

private val chainConfigCache: Map<Chain, ChainConfig> by lazy {
    Chain.entries.associateWith { gemConfig.getChainConfig(it.string) }
}

fun Chain.chainConfig(): ChainConfig = chainConfigCache.getValue(this)

fun Chain.assetType(): AssetType? = chainConfig().defaultAssetType?.toPrimitives()

fun Chain.isStakeSupported(): Boolean = chainConfig().isStakeSupported

fun Chain.asset(): Asset {
    return chainAsset().asset
}

fun Chain.networkName(): String {
    return chainAsset().networkName
}

fun Chain.Companion.available() = Chain.entries.toSet()



fun Chain.isMemoSupport() = chainConfig().isMemoSupported

fun uniffi.gemstone.Chain.requireChain(): Chain = requireNotNull(Chain.entries.firstOrNull { it.string == this }) { "unknown chain: $this" }
