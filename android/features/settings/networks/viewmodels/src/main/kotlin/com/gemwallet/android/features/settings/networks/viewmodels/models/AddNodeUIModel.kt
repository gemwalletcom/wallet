package com.gemwallet.android.features.settings.networks.viewmodels.models

import com.gemwallet.android.model.NodeStatus
import com.wallet.core.primitives.Chain

data class AddNodeUIModel(
    val chain: Chain? = null,
    val status: NodeStatus? = null,
    val checking: Boolean = false,
    val errorResId: Int? = null,
) {
    val canImport: Boolean
        get() = status != null && errorResId == null && !checking
}
