package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequest
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.ConfirmParams.TransferParams.Generic
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.toModel
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.fromJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.swap.ApprovalData
import com.wallet.core.primitives.TransferDataOutputType as PrimitiveOutputType
import uniffi.gemstone.EvmTransactionKind
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.TransferDataOutputType
import uniffi.gemstone.WalletConnectTransaction
import java.math.BigInteger

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
        private val explorerName: String?,
    ) : WCRequest(request), WalletConnectReviewModel {
        val signer: MessageSigner by lazy { MessageSigner(request.message) }

        private val payloadPreview by lazy {
            runCatching { signer.payloadPreview(simulation.payload.map { it.toJson() }) }.getOrNull()
        }

        override val message: String
            get() = runCatching { signer.plainPreview() }.getOrNull() ?: request.message.data.decodeToString()

        override val warnings: List<SimulationWarning>
            get() = simulation.warnings

        override val primaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.primary
                ?.mapNotNull { it.fromJson<SimulationPayloadField>() }
                .orEmpty()
                .withExplorerLinks(chain, explorerName)
        }

        override val secondaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.secondary
                ?.mapNotNull { it.fromJson<SimulationPayloadField>() }
                .orEmpty()
                .withExplorerLinks(chain, explorerName)
        }
    }

    class Transaction(
        private val request: WalletConnectPendingRequest.Transaction,
    ) : WCRequest(request) {
        val isSendable: Boolean get() = request.isSendable

        val inputType: ConfirmParams.TransferParams.InputType
            get() = if (isSendable) ConfirmParams.TransferParams.InputType.EncodeTransaction else ConfirmParams.TransferParams.InputType.Signature

        val confirmParams: Generic
            get() = request.transaction.map(this, isSendable)
    }
}

private fun WalletConnectTransaction.map(
    request: WCRequest.Transaction,
    isSendable: Boolean,
): Generic {
    return when (this) {
        is WalletConnectTransaction.Ethereum -> Generic(
            asset = request.chain.asset(),
            from = request.account,
            metadata = request.appMetadata,
            data = data.data.orEmpty(),
            gasLimit = data.gasLimit,
            inputType = request.inputType,
            destination = DestinationAddress(data.to),
            amount = data.value?.hexToBigInteger() ?: BigInteger.ZERO,
            isSendable = isSendable,
            decodedTransactionType = kind.transactionType,
            approval = kind.approvalData,
        )
        is WalletConnectTransaction.Solana ->
            buildEncodedTransactionParams(request, data.transaction, outputType, isSendable)
        is WalletConnectTransaction.Sui ->
            buildEncodedTransactionParams(request, data.transaction, outputType, isSendable)
        is WalletConnectTransaction.Ton ->
            buildEncodedTransactionParams(request, data, outputType, isSendable)
        is WalletConnectTransaction.Tron ->
            buildEncodedTransactionParams(request, data, outputType, isSendable)
    }
}

private val EvmTransactionKind.transactionType: TransactionType
    get() = when (this) {
        EvmTransactionKind.Transfer -> TransactionType.Transfer
        EvmTransactionKind.ContractCall -> TransactionType.SmartContractCall
        is EvmTransactionKind.TokenApproval -> TransactionType.TokenApproval
    }

private val EvmTransactionKind.approvalData: ApprovalData?
    get() = when (this) {
        EvmTransactionKind.Transfer,
        EvmTransactionKind.ContractCall,
        -> null
        is EvmTransactionKind.TokenApproval -> approval.toModel()
    }

private fun buildEncodedTransactionParams(
    request: WCRequest.Transaction,
    encodedTransaction: String,
    outputType: TransferDataOutputType,
    isSendable: Boolean,
): Generic = Generic(
    asset = request.chain.asset(),
    from = request.account,
    metadata = request.appMetadata,
    data = encodedTransaction,
    gasLimit = null,
    inputType = when (outputType.decodeJson<PrimitiveOutputType>()) {
        PrimitiveOutputType.EncodedTransaction -> ConfirmParams.TransferParams.InputType.EncodeTransaction
        PrimitiveOutputType.Signature -> ConfirmParams.TransferParams.InputType.Signature
    },
    destination = DestinationAddress(""),
    amount = BigInteger.ZERO,
    isSendable = isSendable,
)
