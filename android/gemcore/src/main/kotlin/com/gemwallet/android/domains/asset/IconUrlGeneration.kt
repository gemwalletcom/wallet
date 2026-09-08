package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.gemwallet.android.ext.chainConfig
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemAssetIconImage
import uniffi.gemstone.GemImage
import java.util.concurrent.ConcurrentHashMap
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

private val assetIcons = ConcurrentHashMap<String, GemAssetIcon>()
private val validatorIcons = ConcurrentHashMap<String, String>()
private val nftImages = ConcurrentHashMap<String, String>()

private fun AssetId.icon(): GemAssetIcon = assetIcons.computeIfAbsent(toIdentifier(), assetConfig::assetIcon)

fun Chain.iconChain(): Chain = chainConfig().iconChain.toChain()

fun AssetId.iconChain(): Chain? = when (val image = icon().image) {
    is GemAssetIconImage.Local -> image.chain.toChain()
    is GemAssetIconImage.Remote -> null
}

fun AssetId.supportIconChain(): Chain? = icon().badge?.toChain()

fun AssetId.remoteIconUrl(): String? = when (val image = icon().image) {
    is GemAssetIconImage.Local -> null
    is GemAssetIconImage.Remote -> image.url
}

fun DelegationValidator.getIconUrl(): String =
    validatorIcons.computeIfAbsent("${chain.string}/$id") { GemImage.Validator(chain.string, id).url() }

fun FiatProvider.providerName(): FiatProviderName? = FiatProviderName.entries.firstOrNull { it.string == id.lowercase() }

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()

fun NFTAsset.getImageUrl(): String = nftImageUrl(id.toIdentifier())

fun TransactionNFTTransferMetadata.getImageUrl(): String = nftImageUrl(assetId.toIdentifier())

private fun nftImageUrl(identifier: String): String = nftImages.computeIfAbsent(identifier) { GemImage.NftAsset(it).url() }
