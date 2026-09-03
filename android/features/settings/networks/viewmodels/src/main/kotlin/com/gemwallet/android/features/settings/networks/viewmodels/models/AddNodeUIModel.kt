package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNodeCheck

data class AddNodeUIModel(
    val chain: Chain? = null,
    val status: GemNodeCheck? = null,
    val checking: Boolean = false,
    val errorResId: Int? = null,
) {
    val canImport: Boolean
        get() = status != null && errorResId == null && !checking

    val buttonState: ButtonState
        get() = buttonState(enabled = canImport, loading = checking)
}
