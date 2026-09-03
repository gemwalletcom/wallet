package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemSignMessagePreview
import uniffi.gemstone.GemSignMessageServiceInterface
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.TransferDataOutputAction
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.SignMessage as GemSignMessage

sealed class WCRequest(
    internal val pending: WalletConnectPendingRequest,
) {
    val wallet: Wallet get() = pending.wallet
    val account: Account get() = pending.account
    val appMetadata: ApplicationMetadata get() = pending.appMetadata
    val simulation: SimulationResult get() = pending.simulation
    val name: String get() = appMetadata.shortName
    val icon: String get() = appMetadata.icon
    val description: String get() = appMetadata.description
    val url: String get() = appMetadata.url
    val uri: String get() = url.getShortUrl() ?: url
    val chain: Chain get() = pending.chain

    fun approve(result: String) = pending.approve(result)

    fun reject() = pending.reject()

    class SignMessage(
        private val request: WalletConnectPendingRequest.SignMessage,
        private val service: GemSignMessageServiceInterface,
        override val addressNames: Map<String, String> = emptyMap(),
    ) : WCRequest(request), WalletConnectReviewModel {
        val signMessage: GemSignMessage get() = request.message

        private val preview: GemSignMessagePreview by lazy { service.preview(request.message, simulation.toJson()) }

        override val message: String
            get() = preview.text

        override val warnings: List<SimulationWarning>
            get() = simulation.warnings

        override val primaryPayloadFields: List<PayloadField> by lazy { preview.primaryFields.fields() }

        override val secondaryPayloadFields: List<PayloadField> by lazy { preview.secondaryFields.fields() }

        suspend fun addressNames(): Map<String, String> = service.addressNames(chain.string, preview)
            .map { it.decodeJson<AddressName>() }
            .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
            .associate { it.address.lowercase() to it.name }

        fun withAddressNames(addressNames: Map<String, String>): SignMessage = SignMessage(request, service, addressNames)

        private fun List<uniffi.gemstone.SimulationPayloadField>.fields(): List<PayloadField> =
            map { it.toPrimitives() }.withExplorerLinks(chain) { chain, address -> service.addressUrl(chain.string, address) }
    }

    class Transaction(
        private val request: WalletConnectPendingRequest.Transaction,
    ) : WCRequest(request) {
        val isSendable: Boolean get() = request.isSendable

        val outputAction: TransferDataOutputAction
            get() = if (isSendable) TransferDataOutputAction.Send else TransferDataOutputAction.Sign

        val transfer: GemTransferData
            get() = request.transfer
    }
}
