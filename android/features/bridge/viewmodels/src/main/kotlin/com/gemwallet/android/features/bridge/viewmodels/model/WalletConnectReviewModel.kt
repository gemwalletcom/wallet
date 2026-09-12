package com.gemwallet.android.features.bridge.viewmodels.model

import uniffi.gemstone.GemSimulationValue
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.MessageType

interface WalletConnectReviewModel {
    val icon: String?
    val name: String
    val uri: String
    val chain: Chain
    val primaryPayloadFields: List<PayloadField>
    val secondaryPayloadFields: List<PayloadField>
    val messageType: MessageType
    val message: String
    val warnings: List<GemSimulationWarningRow> get() = emptyList()
    val hasCriticalWarning: Boolean get() = false
    val header: GemSimulationValue? get() = null
    val addressNames: Map<String, String> get() = emptyMap()
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
}
