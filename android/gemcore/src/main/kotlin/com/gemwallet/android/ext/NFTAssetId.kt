package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTCollectionId

private const val TOKEN_ID_SEPARATOR = "::"

fun NFTAssetId.toIdentifier(): String =
    "${chain.string}_$contractAddress$TOKEN_ID_SEPARATOR$tokenId"

val NFTAssetId.identifier: String get() = toIdentifier()

val NFTAssetId.collectionId: NFTCollectionId
    get() = NFTCollectionId(chain = chain, contractAddress = contractAddress)

fun NFTAsset.toNftAssetId(): NFTAssetId =
    NFTAssetId(chain = chain, contractAddress = contractAddress.orEmpty(), tokenId = tokenId)

fun String.toNftAssetId(): NFTAssetId? {
    val chainSeparator = indexOf('_').takeIf { it >= 0 } ?: return null
    val chainRaw = substring(0, chainSeparator)
    val chain = Chain.entries.firstOrNull { it.string == chainRaw } ?: return null
    val rest = substring(chainSeparator + 1)
    val tokenSeparator = rest.indexOf(TOKEN_ID_SEPARATOR).takeIf { it >= 0 } ?: return null
    val contractAddress = rest.substring(0, tokenSeparator)
    val tokenId = rest.substring(tokenSeparator + TOKEN_ID_SEPARATOR.length)
    if (contractAddress.isEmpty() || tokenId.isEmpty()) return null
    return NFTAssetId(chain = chain, contractAddress = contractAddress, tokenId = tokenId)
}

fun NFTCollectionId.toIdentifier(): String = "${chain.string}_$contractAddress"

val NFTCollectionId.identifier: String get() = toIdentifier()

fun NFTCollection.toNftCollectionId(): NFTCollectionId =
    NFTCollectionId(chain = chain, contractAddress = contractAddress)

fun String.toNftCollectionId(): NFTCollectionId? {
    val separator = indexOf('_').takeIf { it >= 0 } ?: return null
    val chainRaw = substring(0, separator)
    val chain = Chain.entries.firstOrNull { it.string == chainRaw } ?: return null
    val contractAddress = substring(separator + 1)
    if (contractAddress.isEmpty()) return null
    return NFTCollectionId(chain = chain, contractAddress = contractAddress)
}
