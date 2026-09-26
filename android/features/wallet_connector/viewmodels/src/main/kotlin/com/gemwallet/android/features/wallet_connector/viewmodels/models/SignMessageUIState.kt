package com.gemwallet.android.features.wallet_connector.viewmodels.models

import android.content.Context
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSignMessagePreview
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.SignMessage as GemSignMessage

class SignMessageUIState(
    private val request: WalletConnectPendingRequest.SignMessage,
    private val row: GemConnectionRow,
    private val service: GemSignMessageServiceInterface,
    private val texts: ReviewTexts,
    private val context: Context,
    private val namedPreview: GemSignMessagePreview? = null,
) : WalletConnectReviewModel {
    val wallet: Wallet get() = request.wallet

    override val chain: Chain get() = request.chain

    override val name: String get() = row.title

    override val icon: String? get() = row.iconUrl

    override val uri: String get() = row.host.orEmpty()

    val signMessage: GemSignMessage get() = request.message

    override val viewFullMessageListItem: ListItemModel get() = ListItemModel(title = texts.viewFullMessage)

    private val preview: GemSignMessagePreview by lazy { namedPreview ?: service.preview(request.request) }

    val rows: List<GemListRow> get() = preview.rows

    override val title: GemLocalizedText get() = preview.title

    override val message: String
        get() = preview.text

    override val warnings: List<GemListRow>
        get() = preview.warnings

    override val hasCriticalWarning: Boolean
        get() = preview.hasCriticalWarning

    override val header: SimulationHeaderUIModel?
        get() = preview.header?.headerUIModel(context)

    override val primaryPayloadFields: List<GemSimulationPayloadRow> get() = preview.primaryFields

    override val secondaryPayloadFields: List<GemSimulationPayloadRow> get() = preview.secondaryFields

    suspend fun withAddressNames(): SignMessageUIState = SignMessageUIState(request, row, service, texts, context, service.withAddressNames(chain.string, preview))
}
