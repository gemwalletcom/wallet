package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Banner
import uniffi.gemstone.GemBannerContent

interface GetBannerContent {
    operator fun invoke(banner: Banner): GemBannerContent
}
