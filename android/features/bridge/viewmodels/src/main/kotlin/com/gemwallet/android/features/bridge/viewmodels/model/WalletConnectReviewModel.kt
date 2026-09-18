package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.MessageType

interface WalletConnectReviewModel {
    val appListItem: ListItemModel
    val walletListItem: ListItemModel
    val viewFullMessageListItem: ListItemModel
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
    val header: SimulationHeaderUIModel? get() = null
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
}
