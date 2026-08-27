package com.gemwallet.android.domains.banner

import com.wallet.core.primitives.BannerEvent

sealed interface BannerAction {
    data class Event(val event: BannerEvent) : BannerAction
    data object Close : BannerAction
}
