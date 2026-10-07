package com.gemwallet.android.ext

import com.wallet.core.primitives.AssetId

fun AssetId.toIdentifier() = "${chain.string}${if (tokenId.isNullOrEmpty()) "" else "_$tokenId"}"

fun String.toAssetId(): AssetId? = runCatching { AssetId(this) }.getOrNull()
