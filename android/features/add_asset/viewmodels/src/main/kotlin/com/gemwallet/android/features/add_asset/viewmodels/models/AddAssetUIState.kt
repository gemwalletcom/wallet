package com.gemwallet.android.features.add_asset.viewmodels.models

class AddAssetUIState(
    val scene: Scene = Scene.Form,
) {
    enum class Scene {
        QrScanner,
        Form,
        SelectChain,
    }
}

sealed interface TokenSearchState {
    object Idle : TokenSearchState
    object Loading : TokenSearchState
    data object Error : TokenSearchState
}