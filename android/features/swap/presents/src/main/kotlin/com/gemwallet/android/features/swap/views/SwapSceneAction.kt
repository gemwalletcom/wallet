package com.gemwallet.android.features.swap.views

import com.gemwallet.android.domains.swap.SwapItemType

internal sealed interface SwapSceneAction {
    data class SelectAsset(val type: SwapItemType) : SwapSceneAction
    data object SwitchAssets : SwapSceneAction
    data object ShowDetails : SwapSceneAction
    data object Swap : SwapSceneAction
    data object Cancel : SwapSceneAction
}
