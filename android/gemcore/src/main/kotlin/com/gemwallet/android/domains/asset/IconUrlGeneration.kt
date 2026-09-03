package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.twoSubtokenIds
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import uniffi.gemstone.Config
import uniffi.gemstone.GemImage
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.SwapperProvider

//fun Int.getDrawableUri() = "android.resource://com.gemwallet.android/drawable/$this"

fun Chain.getIconUrl(): String = chainIconUrl(iconChain())

fun Chain.iconChain(): Chain {
    val icon = Config().getChainConfig(string).iconChain
    return Chain.entries.firstOrNull { it.string == icon } ?: this
}

fun Chain.badgeChain(): Chain? {
    val badge = Config().getChainConfig(string).badgeChain ?: return null
    return Chain.entries.firstOrNull { it.string == badge }
}

private fun chainIconUrl(chain: Chain): String = "file:///android_asset/chains/icons/${chain.string}.svg"

fun AssetId.getIconUrl(): String = when {
    tokenId.isNullOrEmpty() -> chain.getIconUrl()
    else -> GemImage.Asset(toIdentifier()).url()
}

fun AssetId.getSupportIconUrl(): String? = when (type()) {
    AssetSubtype.NATIVE -> chain.badgeChain()?.let { chainIconUrl(it) }
    AssetSubtype.TOKEN -> chain.getIconUrl()
}

fun Asset.getIconUrl(): String {
    if (type == AssetType.PERPETUAL) {
        val chainIcon = id.twoSubtokenIds()?.second?.let { symbol ->
            Chain.entries.firstOrNull { it.asset().symbol == symbol }?.getIconUrl()
        }

        if (chainIcon != null) return chainIcon
    }
    return id.getIconUrl()
}

fun Asset.getSupportIconUrl(): String? = id.getSupportIconUrl()

fun DelegationValidator.getIconUrl(): String = GemImage.Validator(chain.string, id).url()

fun FiatProviderName.getFiatProviderIcon(): String = "file:///android_asset/fiat/${string}.svg"

fun FiatProvider.getFiatProviderIcon(): String = "file:///android_asset/fiat/${id.lowercase()}.svg"

fun SwapperProvider.getSwapProviderIcon(): String {
    val iconName = when (this) {
        SwapperProvider.UNISWAP_V4,
        SwapperProvider.UNISWAP_V3 -> "uniswap"
        SwapperProvider.PANCAKESWAP_V3 -> "pancakeswap"
        SwapperProvider.THORCHAIN -> "thorchain"
        SwapperProvider.MAYACHAIN -> return Chain.Mayachain.getIconUrl()
        SwapperProvider.JUPITER -> "jupiter"
        SwapperProvider.ACROSS -> "across"
        SwapperProvider.OKU -> "oku"
        SwapperProvider.WAGMI -> "wagmi"
        SwapperProvider.CETUS_AGGREGATOR,
        SwapperProvider.CETUS_CLMM -> "cetus"
        SwapperProvider.STONFI_V2 -> "stonfi"
        SwapperProvider.MAYAN -> "mayan"
        SwapperProvider.CHAINFLIP -> "chainflip"
        SwapperProvider.RELAY -> "relay"
        SwapperProvider.AERODROME -> "aerodrome"
        SwapperProvider.HYPERLIQUID -> "hyperliquid"
        SwapperProvider.NEAR_INTENTS -> "near"
        SwapperProvider.ORCA -> "orca"
        SwapperProvider.PANORA -> "panora"
        SwapperProvider.OKX -> "okx"
        SwapperProvider.SQUID -> "squid"
        SwapperProvider.SWAPS_XYZ -> "swaps_xyz"
    }
    return "file:///android_asset/swap/${iconName.lowercase()}.svg"
}

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()

fun NFTAsset.getImageUrl(): String = GemImage.NftAsset(id.toIdentifier()).url()

fun TransactionNFTTransferMetadata.getImageUrl(): String = GemImage.NftAsset(assetId.toIdentifier()).url()
