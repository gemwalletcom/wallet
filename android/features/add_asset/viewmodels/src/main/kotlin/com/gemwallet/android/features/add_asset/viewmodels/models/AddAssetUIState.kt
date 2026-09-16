package com.gemwallet.android.features.add_asset.viewmodels.models

import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemErrorText

class AddAssetUIState(
    val scene: Scene = Scene.Form,
    val isLoading: Boolean = false,
    val error: GemErrorText? = null,
) {
    enum class Scene {
        QrScanner,
        Form,
        SelectChain,
    }
}

sealed interface TokenSearchState {
    data object Idle : TokenSearchState
    data object Loading : TokenSearchState
    data class Found(val asset: Asset) : TokenSearchState
    data object Error : TokenSearchState
}
