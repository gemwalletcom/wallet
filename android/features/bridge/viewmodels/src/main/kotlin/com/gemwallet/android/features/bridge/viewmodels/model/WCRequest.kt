package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.ConfirmParams.TransferParams.Generic
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.toModel
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import uniffi.gemstone.MessageSigner
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.serializer.fromJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import uniffi.gemstone.EvmTransactionKind
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.TransferDataOutputType as PrimitiveOutputType
import uniffi.gemstone.TransferDataOutputType
import uniffi.gemstone.WalletConnect
import uniffi.gemstone.WalletConnectAction
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectTransaction
import uniffi.gemstone.WalletConnectTransactionType
import java.math.BigInteger

sealed class WCRequest(
    internal val sessionRequest: WalletConnectSessionRequest,
    internal val account: Account,
    internal val appMetadata: ApplicationMetadata,
) {
    internal val walletConnect = WalletConnect()

    val requestId: Long get() = sessionRequest.request.id

    val topic: String get() = sessionRequest.topic

    val name: String get() = appMetadata.shortName
    val icon: String get() = appMetadata.icon
    val description: String get() = appMetadata.description
    val url: String get() = appMetadata.url
    val uri: String get() = url.getShortUrl() ?: url

    val chain: Chain get() = account.chain

    class SignMessage(
        sessionRequest: WalletConnectSessionRequest,
        account: Account,
        appMetadata: ApplicationMetadata,
        val action: WalletConnectAction.SignMessage,
        val simulation: SimulationResult,
        private val explorerName: String?,
    ) : WCRequest(sessionRequest, account, appMetadata), WalletConnectReviewModel {

        private val signer by lazy {
            runCatching {
                MessageSigner(walletConnect.decodeSignMessage(action.chain, action.signType, action.data))
            }
        }

        private val payloadPreview by lazy {
            signer.getOrNull()?.let { signer ->
                runCatching { signer.payloadPreview(simulation.payload.map { it.toJson() }) }.getOrNull()
            }
        }

        override val message: String
            get() = signer.getOrNull()?.plainPreview() ?: action.data

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

        suspend fun execute(
            signMessageOperator: GemSignMessageOperator,
            wallet: com.wallet.core.primitives.Wallet,
            password: String,
        ): String {
            val signature = signMessageOperator.sign(signer.getOrThrow(), wallet, password)
            return walletConnect.encodeSignMessage(chain.string, signature).payload()
        }
    }

    abstract class Transaction(
        sessionRequest: WalletConnectSessionRequest,
        account: Account,
        appMetadata: ApplicationMetadata,
        val isSendable: Boolean,
        val inputType: ConfirmParams.TransferParams.InputType,
        val transactionType: WalletConnectTransactionType,
        val data: String,
        val simulation: SimulationResult,
    ) : WCRequest(sessionRequest, account, appMetadata) {

        open val confirmParams: Generic
            get() = walletConnect.decodeSendTransaction(transactionType, data).map(this, isSendable)

        abstract fun execute(result: String): String

        abstract class Signing(
            sessionRequest: WalletConnectSessionRequest,
            account: Account,
            appMetadata: ApplicationMetadata,
            transactionType: WalletConnectTransactionType,
            data: String,
            simulation: SimulationResult,
        ) : Transaction(
            sessionRequest = sessionRequest,
            account = account,
            appMetadata = appMetadata,
            isSendable = false,
            inputType = ConfirmParams.TransferParams.InputType.Signature,
            transactionType = transactionType,
            data = data,
            simulation = simulation,
        )

        class SignTransaction(
            sessionRequest: WalletConnectSessionRequest,
            account: Account,
            appMetadata: ApplicationMetadata,
            val action: WalletConnectAction.SignTransaction,
            simulation: SimulationResult,
        ) : Signing(
            sessionRequest = sessionRequest,
            account = account,
            appMetadata = appMetadata,
            transactionType = action.transactionType,
            data = action.data,
            simulation = simulation,
        ) {

            override fun execute(result: String): String =
                walletConnect.encodeSignTransaction(action.chain, result).payload()
        }

        class SignAllTransactions(
            sessionRequest: WalletConnectSessionRequest,
            account: Account,
            appMetadata: ApplicationMetadata,
            transactionType: WalletConnectTransactionType,
            data: String,
            simulation: SimulationResult,
        ) : Signing(
            sessionRequest = sessionRequest,
            account = account,
            appMetadata = appMetadata,
            transactionType = transactionType,
            data = data,
            simulation = simulation,
        ) {

            override fun execute(result: String): String =
                walletConnect.encodeSignAllTransactions(listOf(result)).payload()
        }

        class SendTransaction(
            sessionRequest: WalletConnectSessionRequest,
            account: Account,
            appMetadata: ApplicationMetadata,
            val action: WalletConnectAction.SendTransaction,
            simulation: SimulationResult,
        ) : Transaction(
            sessionRequest = sessionRequest,
            account = account,
            appMetadata = appMetadata,
            isSendable = true,
            inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
            transactionType = action.transactionType,
            data = action.data,
            simulation = simulation,
        ) {

            override fun execute(result: String): String =
                walletConnect.encodeSendTransaction(action.chain, result).payload()
        }
    }
}

internal fun WalletConnectResponseType.payload(): String = when (this) {
    is WalletConnectResponseType.Object -> json
    is WalletConnectResponseType.String -> value
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
