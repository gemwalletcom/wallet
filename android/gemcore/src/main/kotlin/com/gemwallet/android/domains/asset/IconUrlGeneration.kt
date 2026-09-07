package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import uniffi.gemstone.Config
import uniffi.gemstone.GemAssetIconImage
import uniffi.gemstone.GemImage
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

fun Chain.iconChain(): Chain = Config().getChainConfig(string).iconChain.toChain()

fun AssetId.iconChain(): Chain? = when (val image = assetConfig.assetIcon(toIdentifier()).image) {
    is GemAssetIconImage.Local -> image.chain.toChain()
    is GemAssetIconImage.Remote -> null
}

fun AssetId.supportIconChain(): Chain? = assetConfig.assetIcon(toIdentifier()).badge?.toChain()

fun AssetId.remoteIconUrl(): String? = when (val image = assetConfig.assetIcon(toIdentifier()).image) {
    is GemAssetIconImage.Local -> null
    is GemAssetIconImage.Remote -> image.url
}

fun DelegationValidator.getIconUrl(): String = GemImage.Validator(chain.string, id).url()

fun FiatProvider.providerName(): FiatProviderName? = FiatProviderName.entries.firstOrNull { it.string == id.lowercase() }

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()

fun NFTAsset.getImageUrl(): String = GemImage.NftAsset(id.toIdentifier()).url()

fun TransactionNFTTransferMetadata.getImageUrl(): String = GemImage.NftAsset(assetId.toIdentifier()).url()
