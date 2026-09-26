package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.chainConfig
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.GemImage
import java.util.concurrent.ConcurrentHashMap

private val nftImages = ConcurrentHashMap<String, String>()

fun Chain.iconChain(): Chain = chainConfig().iconChain.toChain()

fun getListIconUrl(listId: String): String = GemImage.AssetList(listId).url()

fun TransactionNFTTransferMetadata.getImageUrl(): String = nftImageUrl(assetId.toIdentifier())

private fun nftImageUrl(identifier: String): String = nftImages.computeIfAbsent(identifier) { GemImage.NftAsset(it).url() }
