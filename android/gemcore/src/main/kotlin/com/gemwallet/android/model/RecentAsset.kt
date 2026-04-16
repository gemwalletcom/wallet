package com.gemwallet.android.model

import com.wallet.core.primitives.Asset

data class RecentAsset(
    val asset: Asset,
    val addedAt: Long,
)
