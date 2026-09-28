package com.gemwallet.android.features.swap.presents

import com.gemwallet.android.domains.swap.SwapItemType

internal sealed interface SwapAction {
    data class SelectAsset(val type: SwapItemType) : SwapAction
    data class SelectPercent(val percent: Int) : SwapAction
    data object SwitchAssets : SwapAction
    data object ShowDetails : SwapAction
    data object Slippage : SwapAction
    data object Swap : SwapAction
    data object Cancel : SwapAction
}
