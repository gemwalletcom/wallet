package com.gemwallet.android.features.assets.presents.select

import com.wallet.core.primitives.Asset

internal sealed interface RecentsAction {
    data object Dismiss : RecentsAction
    data object Clear : RecentsAction
    data class Select(val asset: Asset) : RecentsAction
}
