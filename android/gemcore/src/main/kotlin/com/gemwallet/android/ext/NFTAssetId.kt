package com.gemwallet.android.ext

import com.wallet.core.primitives.NFTAssetId

private const val TOKEN_ID_SEPARATOR = "::"

fun NFTAssetId.toIdentifier(): String =
    "${chain.string}_$contractAddress$TOKEN_ID_SEPARATOR$tokenId"

fun String.toNftAssetId(): NFTAssetId? = runCatching { NFTAssetId(this) }.getOrNull()
