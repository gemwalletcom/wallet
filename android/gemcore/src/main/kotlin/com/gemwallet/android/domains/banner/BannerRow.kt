package com.gemwallet.android.domains.banner

import com.wallet.core.primitives.Banner
import uniffi.gemstone.GemBannerContent

data class BannerRow(
    val banner: Banner,
    val content: GemBannerContent,
)
