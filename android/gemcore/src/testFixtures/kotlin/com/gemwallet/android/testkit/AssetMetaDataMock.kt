package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetMetaData

fun mockAssetMetaData(
    isSellEnabled: Boolean = false,
    isStakeEnabled: Boolean = false,
    isPinned: Boolean = false,
    stakingApr: Double? = null,
) = AssetMetaData(
    isEnabled = true,
    isBalanceEnabled = true,
    isBuyEnabled = false,
    isSellEnabled = isSellEnabled,
    isSwapEnabled = false,
    isStakeEnabled = isStakeEnabled,
    isEarnEnabled = false,
    isPinned = isPinned,
    isActive = true,
    rankScore = 1,
    stakingApr = stakingApr,
    earnApr = null,
)
