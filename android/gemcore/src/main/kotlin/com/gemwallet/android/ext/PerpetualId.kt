package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualId

fun PerpetualId.toIdentifier(): String = "${provider.string}_$symbol"

fun String.toPerpetualId(): PerpetualId? = runCatching { PerpetualId(this) }.getOrNull()
