package com.gemwallet.android.ext

import com.wallet.core.primitives.NFTCollectionId

fun NFTCollectionId.toIdentifier(): String = "${chain.string}_$contractAddress"

fun String.toNftCollectionId(): NFTCollectionId? = runCatching { NFTCollectionId(this) }.getOrNull()
