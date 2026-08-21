package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.toDTO
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAsset
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.EVMChain
import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.Config
import uniffi.gemstone.supportsPrivateKeyImport
import uniffi.gemstone.validateAddress
import uniffi.gemstone.checksumAddress as gemstoneChecksumAddress
import java.math.BigInteger

private val chainAssetCache: Map<Chain, ChainAsset> by lazy {
    Chain.entries.associateWith { chain ->
        val wrapper = uniffi.gemstone.chainAssetWrapper(chain.string)
        ChainAsset(
            asset = wrapper.asset.toDTO(),
            networkName = wrapper.networkName,
        )
    }
}

private fun Chain.chainAsset(): ChainAsset {
    return chainAssetCache[this] ?: throw IllegalArgumentException("Unsupported chain: $string")
}

fun Chain.assetType(): AssetType? {
    return when (this) {
        Chain.OpBNB,
        Chain.SmartChain -> AssetType.BEP20

        Chain.Tron -> AssetType.TRC20

        Chain.Solana -> AssetType.SPL

        Chain.Ton -> AssetType.JETTON

        Chain.Aptos,
        Chain.Sui -> AssetType.TOKEN

        Chain.Tempo -> AssetType.TIP20

        Chain.Ethereum,
        Chain.Polygon,
        Chain.Arbitrum,
        Chain.Optimism,
        Chain.Base,
        Chain.AvalancheC,
        Chain.Fantom,
        Chain.Gnosis,
        Chain.Manta,
        Chain.Blast,
        Chain.ZkSync,
        Chain.Linea,
        Chain.Mantle,
        Chain.Sonic,
        Chain.SeiEvm,
        Chain.Abstract,
        Chain.Ink,
        Chain.Berachain,
        Chain.Unichain,
        Chain.Hyperliquid,
        Chain.Monad,
        Chain.Plasma,
        Chain.XLayer,
        Chain.Stable,
        Chain.Robinhood,
        Chain.World -> AssetType.ERC20

        Chain.HyperCore,
        Chain.Cosmos,
        Chain.Osmosis,
        Chain.Celestia,
        Chain.Injective,
        Chain.Sei,
        Chain.Noble,
        Chain.Celo,
        Chain.Bitcoin,
        Chain.BitcoinCash,
        Chain.Litecoin,
        Chain.Doge,
        Chain.Thorchain,
        Chain.Xrp,
        Chain.Algorand,
        Chain.Stellar,
        Chain.Polkadot,
        Chain.Cardano,
        Chain.Zcash,
        Chain.Near,
        Chain.Mayachain -> null
    }
}

fun Chain.toEVM(): EVMChain? {
    return EVMChain.entries.firstOrNull { it.string == string }
}

fun Chain.getReserveBalance(): BigInteger = Config().getChainConfig(this.string).accountActivationFee?.toBigInteger() ?: BigInteger.ZERO

fun Chain.getReserveBalanceUrl(): String? = Config().getChainConfig(this.string).accountActivationFeeUrl

fun Chain.isStakeSupported(): Boolean = Config().getChainConfig(this.string).isStakeSupported

fun Chain.isNftSupported(): Boolean = Config().getChainConfig(this.string).isNftSupported

fun Chain.hasNativeAsset(): Boolean = Config().getChainConfig(this.string).hasNativeAsset

fun Chain.asset(): Asset {
    return chainAsset().asset
}

fun Chain.networkName(): String {
    return chainAsset().networkName
}

fun Chain.Companion.available() = Chain.entries.toSet()

fun List<Chain>.filter(query: String): List<Chain> {
    return if (query.isBlank()) this else filter { chain ->
        val chainAsset = chain.chainAsset()
        chainAsset.networkName.contains(query, ignoreCase = true) ||
            chain.string.contains(query, ignoreCase = true) ||
            chainAsset.asset.name.contains(query, ignoreCase = true) ||
            chainAsset.asset.symbol.contains(query, ignoreCase = true)
    }
}

fun Chain.toChainType(): ChainType {
    return when (this) {
        Chain.HyperCore -> ChainType.HyperCore
        Chain.Solana -> ChainType.Solana
        Chain.Ton -> ChainType.Ton
        Chain.Tron -> ChainType.Tron
        Chain.Aptos -> ChainType.Aptos
        Chain.Sui -> ChainType.Sui
        Chain.Xrp -> ChainType.Xrp
        Chain.Near -> ChainType.Near
        Chain.Stellar -> ChainType.Stellar
        Chain.Algorand -> ChainType.Algorand
        Chain.Polkadot -> ChainType.Polkadot
        Chain.Cardano -> ChainType.Cardano
        Chain.Bitcoin,
        Chain.Doge,
        Chain.BitcoinCash,
        Chain.Zcash,
        Chain.Litecoin -> ChainType.Bitcoin
        Chain.Thorchain,
        Chain.Mayachain,
        Chain.Osmosis,
        Chain.Celestia,
        Chain.Injective,
        Chain.Sei,
        Chain.Noble,
        Chain.Cosmos -> ChainType.Cosmos
        Chain.AvalancheC,
        Chain.Base,
        Chain.SmartChain,
        Chain.Arbitrum,
        Chain.Polygon,
        Chain.OpBNB,
        Chain.Fantom,
        Chain.Gnosis,
        Chain.Optimism,
        Chain.Manta,
        Chain.Blast,
        Chain.ZkSync,
        Chain.Linea,
        Chain.Mantle,
        Chain.Celo,
        Chain.World,
        Chain.Sonic,
        Chain.SeiEvm,
        Chain.Abstract,
        Chain.Berachain,
        Chain.Unichain,
        Chain.Ink,
        Chain.Hyperliquid,
        Chain.Monad,
        Chain.Plasma,
        Chain.XLayer,
        Chain.Stable,
        Chain.Robinhood,
        Chain.Ethereum -> ChainType.Ethereum
        Chain.Tempo -> ChainType.Ethereum
    }
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

val Chain.Companion.referralChain: Chain get() = Chain.Ethereum
