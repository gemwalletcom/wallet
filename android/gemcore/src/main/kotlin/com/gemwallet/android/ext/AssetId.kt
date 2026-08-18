package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype

fun AssetId.toIdentifier() = "${chain.string}${if (tokenId.isNullOrEmpty()) "" else "_${tokenId}"}"

val AssetId.identifier: String get() = "${chain.string}${if (tokenId.isNullOrEmpty()) "" else "_${tokenId}"}"

fun AssetId.type() = if (tokenId.isNullOrEmpty()) AssetSubtype.NATIVE else AssetSubtype.TOKEN

fun String.toAssetId(): AssetId? {
    return runCatching { AssetId(this) }.getOrNull()
}

fun AssetId.twoSubtokenIds(): Pair<String, String>? = tokenId
    ?.split("::")
    ?.takeIf { it.size >= 2 }?.let {
        Pair(it[0], it[1])
    }
