package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAsset
import com.wallet.core.primitives.ChainType
import uniffi.gemstone.Config
import uniffi.gemstone.supportsPrivateKeyImport
import uniffi.gemstone.GemAssetConfigService

private val assetConfig = GemAssetConfigService()

private val chainAssetCache: Map<Chain, ChainAsset> by lazy {
    Chain.entries.associateWith { chain ->
        assetConfig.chainAsset(chain.string).decodeJson()
    }
}

private fun Chain.chainAsset(): ChainAsset {
    return chainAssetCache[this] ?: throw IllegalArgumentException("Unsupported chain: $string")
}

fun Chain.assetType(): AssetType? {
    val defaultAssetType = Config().getChainConfig(string).defaultAssetType ?: return null
    return AssetType.entries.firstOrNull { it.string == defaultAssetType }
}

fun Chain.isStakeSupported(): Boolean = Config().getChainConfig(this.string).isStakeSupported

fun Chain.asset(): Asset {
    return chainAsset().asset
}

fun Chain.networkName(): String {
    return chainAsset().networkName
}

fun Chain.Companion.available() = Chain.entries.toSet()


fun Chain.toChainType(): ChainType {
    val chainType = Config().getChainConfig(string).chainType
    return requireNotNull(ChainType.entries.firstOrNull { it.string == chainType }) { "Unknown chain type: $chainType" }
}



fun Chain.isSwapSupport(): Boolean = Config().getChainConfig(string).isSwapSupported

fun Chain.isMemoSupport() = Config().getChainConfig(string).isMemoSupported

fun Chain.isPrivateKeyImportSupported(): Boolean = supportsPrivateKeyImport(string)

fun uniffi.gemstone.Chain.requireChain(): Chain = requireNotNull(Chain.entries.firstOrNull { it.string == this }) { "unknown chain: $this" }
