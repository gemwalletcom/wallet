package com.gemwallet.android.ext

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Banner
import uniffi.gemstone.GemBannerKey

fun Banner.toGemKey() = GemBannerKey(
    walletId = walletId?.id,
    assetId = asset?.id?.toIdentifier(),
    event = event.toGem(),
)

