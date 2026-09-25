package com.gemwallet.android.features.wallet_connector.viewmodels.model

import android.content.Context
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSignMessagePreview
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.SignMessage as GemSignMessage

sealed class WCRequest(internal val pending: WalletConnectPendingRequest, private val row: GemConnectionRow) {
    val wallet: Wallet get() = pending.wallet
    val account: Account get() = pending.account
    val appMetadata: ApplicationMetadata get() = pending.appMetadata
    val simulation: SimulationResult get() = pending.simulation
    val name: String get() = row.title
    val icon: String? get() = row.iconUrl
    val description: String get() = appMetadata.description
    val url: String get() = appMetadata.url
    val uri: String get() = row.host.orEmpty()
    val chain: Chain get() = pending.chain

    fun approve(result: String) = pending.approve(result)

    fun reject() = pending.reject()

    class SignMessage(
        private val request: WalletConnectPendingRequest.SignMessage,
        private val row: GemConnectionRow,
        private val service: GemSignMessageServiceInterface,
        private val texts: ReviewTexts,
        private val context: Context,
        private val namedPreview: GemSignMessagePreview? = null,
    ) : WCRequest(request, row),
        WalletConnectReviewModel {
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

        suspend fun withAddressNames(): SignMessage = SignMessage(request, row, service, texts, context, service.withAddressNames(chain.string, preview))
    }

    class Transaction(private val request: WalletConnectPendingRequest.Transaction, row: GemConnectionRow) : WCRequest(request, row) {
        val isSendable: Boolean get() = request.isSendable

        val input: ConfirmTransferInput
            get() = ConfirmTransferInput(request.transfer, request.wallet)
    }
}
