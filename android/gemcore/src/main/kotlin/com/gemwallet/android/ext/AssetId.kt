package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype

fun AssetId.toIdentifier() = "${chain.string}${if (tokenId.isNullOrEmpty()) "" else "_${tokenId}"}"

fun AssetId.type() = if (tokenId.isNullOrEmpty()) AssetSubtype.NATIVE else AssetSubtype.TOKEN

fun String.toAssetId(): AssetId? {
    return runCatching { AssetId(this) }.getOrNull()
}

