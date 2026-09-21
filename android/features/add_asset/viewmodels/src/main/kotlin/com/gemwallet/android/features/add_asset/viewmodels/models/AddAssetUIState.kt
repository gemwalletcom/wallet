package com.gemwallet.android.features.add_asset.viewmodels.models

import uniffi.gemstone.GemErrorText

class AddAssetUIState(val scene: Scene = Scene.Form, val isLoading: Boolean = false, val error: GemErrorText? = null) {
    enum class Scene {
        QrScanner,
        Form,
        SelectChain,
    }
}
