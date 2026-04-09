package com.gemwallet.android.application.banner.coordinators

import com.wallet.core.primitives.Banner

interface CancelBanner {
    suspend operator fun invoke(banner: Banner)
}
