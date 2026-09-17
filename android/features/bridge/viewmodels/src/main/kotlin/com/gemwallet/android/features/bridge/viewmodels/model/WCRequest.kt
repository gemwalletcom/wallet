package com.gemwallet.android.features.bridge.viewmodels.model

import android.content.Context
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemSignMessagePreview
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.MessageType
import uniffi.gemstone.SignMessage as GemSignMessage
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.simulationWarningRows

sealed class WCRequest(
    internal val pending: WalletConnectPendingRequest,
    private val row: GemConnectionRow,
) {
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
        override val addressNames: Map<String, String> = emptyMap(),
    ) : WCRequest(request, row), WalletConnectReviewModel {
        val signMessage: GemSignMessage get() = request.message

        override val appListItem: ListItemModel get() = ListItemModel(title = texts.app, subtitle = row.title)
        override val walletListItem: ListItemModel get() = ListItemModel(title = texts.wallet, subtitle = request.wallet.name)
        override val viewFullMessageListItem: ListItemModel get() = ListItemModel(title = texts.viewFullMessage)

        private val preview: GemSignMessagePreview by lazy { service.preview(request.message, simulation, request.assets) }

        override val messageType: MessageType get() = preview.messageType

        override val message: String
            get() = preview.text

        override val warnings: List<GemSimulationWarningRow>
            get() = simulationWarningRows(simulation.warnings)

        override val hasCriticalWarning: Boolean
            get() = preview.hasCriticalWarning

        override val header: SimulationHeaderUIModel?
            get() = preview.header?.headerUIModel(context)

        override val primaryPayloadFields: List<PayloadField> by lazy { preview.primaryFields.fields() }

        override val secondaryPayloadFields: List<PayloadField> by lazy { preview.secondaryFields.fields() }

        suspend fun addressNames(): Map<String, String> = service.addressNames(chain.string, preview)
            .map { it.toPrimitives() }
            .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
            .associate { it.address.lowercase() to it.name }

        fun withAddressNames(addressNames: Map<String, String>): SignMessage = SignMessage(request, row, service, texts, context, addressNames)

        private fun List<uniffi.gemstone.SimulationPayloadField>.fields(): List<PayloadField> =
            withExplorerLinks(chain) { chain, address -> service.addressUrl(chain.string, address) }
    }

    class Transaction(
        private val request: WalletConnectPendingRequest.Transaction,
        row: GemConnectionRow,
    ) : WCRequest(request, row) {
        val isSendable: Boolean get() = request.isSendable

        val input: ConfirmTransferInput
            get() = ConfirmTransferInput(request.transfer)
    }
}
