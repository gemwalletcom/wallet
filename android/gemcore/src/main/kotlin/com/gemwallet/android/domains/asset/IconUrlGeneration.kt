package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Chain
import uniffi.gemstone.Config
import uniffi.gemstone.GemImage
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.SwapProvider

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

fun Asset.getIconUrl(): String = (assetConfig.iconAssetId(id.toIdentifier()).toAssetId() ?: id).getIconUrl()

fun Asset.getSupportIconUrl(): String? = id.getSupportIconUrl()

fun DelegationValidator.getIconUrl(): String = GemImage.Validator(chain.string, id).url()

fun FiatProviderName.getFiatProviderIcon(): String = "file:///android_asset/fiat/${string}.svg"

fun FiatProvider.getFiatProviderIcon(): String = "file:///android_asset/fiat/${id.lowercase()}.svg"

fun SwapProvider.getSwapProviderIcon(): String {
    val iconName = when (this) {
        SwapProvider.UNISWAP_V4,
        SwapProvider.UNISWAP_V3 -> "uniswap"
        SwapProvider.PANCAKESWAP_V3 -> "pancakeswap"
        SwapProvider.THORCHAIN -> "thorchain"
        SwapProvider.MAYACHAIN -> return Chain.Mayachain.getIconUrl()
        SwapProvider.JUPITER -> "jupiter"
        SwapProvider.ACROSS -> "across"
        SwapProvider.OKU -> "oku"
        SwapProvider.WAGMI -> "wagmi"
        SwapProvider.CETUS_AGGREGATOR,
        SwapProvider.CETUS_CLMM -> "cetus"
        SwapProvider.STONFI_V2 -> "stonfi"
        SwapProvider.MAYAN -> "mayan"
        SwapProvider.CHAINFLIP -> "chainflip"
        SwapProvider.RELAY -> "relay"
        SwapProvider.AERODROME -> "aerodrome"
        SwapProvider.HYPERLIQUID -> "hyperliquid"
        SwapProvider.NEAR_INTENTS -> "near"
        SwapProvider.ORCA -> "orca"
        SwapProvider.PANORA -> "panora"
        SwapProvider.OKX -> "okx"
        SwapProvider.SQUID -> "squid"
        SwapProvider.SWAPS_XYZ -> "swaps_xyz"
    }
    return "file:///android_asset/swap/${iconName.lowercase()}.svg"
}

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()

fun NFTAsset.getImageUrl(): String = GemImage.NftAsset(id.toIdentifier()).url()

fun TransactionNFTTransferMetadata.getImageUrl(): String = GemImage.NftAsset(assetId.toIdentifier()).url()
