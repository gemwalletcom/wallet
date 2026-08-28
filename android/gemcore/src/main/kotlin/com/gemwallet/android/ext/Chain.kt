package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAsset
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.EVMChain
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.searchMatchingChains
import uniffi.gemstone.supportsPrivateKeyImport
import uniffi.gemstone.validateAddress
import uniffi.gemstone.checksumAddress as gemstoneChecksumAddress
import java.math.BigInteger

private val chainAssetCache: Map<Chain, ChainAsset> by lazy {
    Chain.entries.associateWith { chain ->
        uniffi.gemstone.chainAssetWrapper(chain.string).decodeJson()
    }
}

private fun Chain.chainAsset(): ChainAsset {
    return chainAssetCache[this] ?: throw IllegalArgumentException("Unsupported chain: $string")
}

fun Chain.assetType(): AssetType? {
    val defaultAssetType = Config().getChainConfig(string).defaultAssetType ?: return null
    return AssetType.entries.firstOrNull { it.string == defaultAssetType }
}

fun Chain.toEVM(): EVMChain? {
    return EVMChain.entries.firstOrNull { it.string == string }
}

fun Chain.getReserveBalance(): BigInteger = Config().getChainConfig(this.string).accountActivationFee?.toBigInteger() ?: BigInteger.ZERO

fun Chain.getReserveBalanceUrl(): String? = Config().getChainConfig(this.string).accountActivationFeeUrl

fun Chain.isStakeSupported(): Boolean = Config().getChainConfig(this.string).isStakeSupported

fun Chain.isNftSupported(): Boolean = Config().getChainConfig(this.string).isNftSupported

fun Chain.supportsNftTransfer(): Boolean = Config().getChainConfig(this.string).supportsNftTransfer

fun Chain.hasNativeAsset(): Boolean = Config().getChainConfig(this.string).hasNativeAsset

fun Chain.asset(): Asset {
    return chainAsset().asset
}

fun Chain.networkName(): String {
    return chainAsset().networkName
}

fun Chain.Companion.available() = Chain.entries.toSet()

fun List<Chain>.filter(query: String): List<Chain> =
    searchMatchingChains(map { it.string }, query).mapNotNull { chainString -> Chain.entries.firstOrNull { it.string == chainString } }

fun Chain.toChainType(): ChainType {
    val chainType = Config().getChainConfig(string).chainType
    return requireNotNull(ChainType.entries.firstOrNull { it.string == chainType }) { "Unknown chain type: $chainType" }
}


fun Chain.getNetworkId(): String {
    return Config().getChainConfig(string).networkId
}

fun Chain.isSwapSupport(): Boolean {
    return try {
        Config().getChainConfig(string).isSwapSupported
    } catch (_: Throwable) {
        false
    }
}

fun Chain.feeUnitType() = FeeUnitType.entries.firstOrNull {
    it.string == Config().getChainConfig(string).feeUnitType
}

fun Chain.isMemoSupport() = Config().getChainConfig(string).isMemoSupported

fun Chain.isValidAddress(address: String): Boolean = validateAddress(checksumAddress(address), string)

fun Chain.checksumAddress(address: String): String = gemstoneChecksumAddress(address = address, chain = string)

fun Chain.isPrivateKeyImportSupported(): Boolean = supportsPrivateKeyImport(string)

fun uniffi.gemstone.Chain.toChain(): Chain? {
    return Chain.entries.firstOrNull { it.string == this }
}

fun uniffi.gemstone.Chain.requireChain(): Chain = requireNotNull(toChain()) { "unknown chain: $this" }

val Chain.Companion.referralChain: Chain get() = Chain.Ethereum
