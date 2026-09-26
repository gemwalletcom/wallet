package com.gemwallet.android.features.assets.viewmodels.add.models

class AddAssetUIState(val scene: Scene = Scene.Form, val isLoading: Boolean = false, val error: String? = null) {
    enum class Scene {
        QrScanner,
        Form,
        SelectChain,
    }
}
