package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.model.AssetValueHeader
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.Chain
import uniffi.gemstone.SimulationWarning
import uniffi.gemstone.MessageType

interface WalletConnectReviewModel {
    val icon: String
    val name: String
    val uri: String
    val chain: Chain
    val primaryPayloadFields: List<PayloadField>
    val secondaryPayloadFields: List<PayloadField>
    val messageType: MessageType
    val message: String
    val warnings: List<SimulationWarning> get() = emptyList()
    val hasCriticalWarning: Boolean get() = false
    val header: AssetValueHeader? get() = null
    val addressNames: Map<String, String> get() = emptyMap()
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
}
