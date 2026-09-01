package com.gemwallet.android.features.bridge.viewmodels.model

import uniffi.gemstone.GemApplicationMetadataService
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.serializer.decodeJsonOrNull
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemExplorerService
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.TransferDataOutputAction
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.MessageSigner

sealed class WCRequest(
    internal val pending: WalletConnectPendingRequest,
    private val applicationMetadataService: GemApplicationMetadataService,
) {
    val wallet: Wallet get() = pending.wallet
    val account: Account get() = pending.account
    val appMetadata: ApplicationMetadata get() = pending.appMetadata
    val simulation: SimulationResult get() = pending.simulation
    val name: String get() = appMetadata.shortName(applicationMetadataService)
    val icon: String get() = appMetadata.icon
    val description: String get() = appMetadata.description
    val url: String get() = appMetadata.url
    val uri: String get() = url.getShortUrl() ?: url
    val chain: Chain get() = pending.chain

    fun approve(result: String) = pending.approve(result)

    fun reject() = pending.reject()

    class SignMessage(
        private val request: WalletConnectPendingRequest.SignMessage,
        private val explorerService: GemExplorerService?,
        applicationMetadataService: GemApplicationMetadataService,
    ) : WCRequest(request, applicationMetadataService), WalletConnectReviewModel {
        val signer: MessageSigner by lazy { MessageSigner(request.message) }

        private val payloadPreview by lazy {
            runCatching { signer.payloadPreview(simulation.payload.map { it.toJson() }) }.getOrNull()
        }

        override val message: String
            get() = runCatching { signer.plainPreview() }.getOrNull() ?: request.message.data.joinToString(separator = "", prefix = "0x") { "%02x".format(it) }

        override val warnings: List<SimulationWarning>
            get() = simulation.warnings

        override val primaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.primary
                ?.mapNotNull { it.decodeJsonOrNull<SimulationPayloadField>() }
                .orEmpty()
                .withExplorerLinks(chain, explorerService)
        }

        override val secondaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.secondary
                ?.mapNotNull { it.decodeJsonOrNull<SimulationPayloadField>() }
                .orEmpty()
                .withExplorerLinks(chain, explorerService)
        }
    }

    class Transaction(
        private val request: WalletConnectPendingRequest.Transaction,
        applicationMetadataService: GemApplicationMetadataService,
    ) : WCRequest(request, applicationMetadataService) {
        val isSendable: Boolean get() = request.isSendable

        val outputAction: TransferDataOutputAction
            get() = if (isSendable) TransferDataOutputAction.Send else TransferDataOutputAction.Sign

        val confirmInput: GemConfirmInput
            get() = request.transfer.confirmInput(account)
    }
}
