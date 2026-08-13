package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationWarning

interface WalletConnectReviewModel {
    val icon: String
    val name: String
    val uri: String
    val chain: Chain
    val primaryPayloadFields: List<PayloadField>
    val secondaryPayloadFields: List<PayloadField>
    val message: String
    val warnings: List<SimulationWarning> get() = emptyList()
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
}
