package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetMetaData

fun mockAssetMetaData(
    isEnabled: Boolean = true,
    isBalanceEnabled: Boolean = true,
    isBuyEnabled: Boolean = false,
    isSellEnabled: Boolean = false,
    isSwapEnabled: Boolean = false,
    isStakeEnabled: Boolean = false,
    isEarnEnabled: Boolean = false,
    isPinned: Boolean = false,
    isActive: Boolean = true,
    rankScore: Int = 1,
    stakingApr: Double? = null,
    earnApr: Double? = null,
) = AssetMetaData(
    isEnabled = isEnabled,
    isBalanceEnabled = isBalanceEnabled,
    isBuyEnabled = isBuyEnabled,
    isSellEnabled = isSellEnabled,
    isSwapEnabled = isSwapEnabled,
    isStakeEnabled = isStakeEnabled,
    isEarnEnabled = isEarnEnabled,
    isPinned = isPinned,
    isActive = isActive,
    rankScore = rankScore,
    stakingApr = stakingApr,
    earnApr = earnApr,
)
