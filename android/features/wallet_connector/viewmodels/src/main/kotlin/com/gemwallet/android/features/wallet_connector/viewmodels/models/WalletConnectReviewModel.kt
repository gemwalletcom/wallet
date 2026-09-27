package com.gemwallet.android.features.wallet_connector.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.GemValueHeader

interface WalletConnectReviewModel {
    val viewFullMessageListItem: ListItemModel
    val icon: String?
    val name: String
    val uri: String
    val chain: Chain
    val primaryPayloadFields: List<GemSimulationPayloadRow>
    val secondaryPayloadFields: List<GemSimulationPayloadRow>
    val title: GemLocalizedText
    val message: String
    val warnings: List<GemListRow> get() = emptyList()
    val hasCriticalWarning: Boolean get() = false
    val header: GemValueHeader? get() = null
    val hasPayload: Boolean get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
}
