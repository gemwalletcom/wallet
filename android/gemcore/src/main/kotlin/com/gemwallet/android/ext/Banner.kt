package com.gemwallet.android.ext

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemBannerKey

fun Banner.toGemKey() = GemBannerKey(
    walletId = walletId?.id,
    assetId = asset?.id?.toIdentifier(),
    event = event.toJson(),
)

fun Wallet.onboardingBannerKey() = GemBannerKey(walletId = id.id, assetId = null, event = BannerEvent.Onboarding.toJson())
