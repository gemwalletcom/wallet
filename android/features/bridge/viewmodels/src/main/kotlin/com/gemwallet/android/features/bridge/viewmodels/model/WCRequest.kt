package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.ConfirmParams.TransferParams.Generic
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.reown.walletkit.client.Wallet
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import uniffi.gemstone.MessageSigner
import com.gemwallet.android.blockchain.gemstone.toGem
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.TransferDataOutputType
import uniffi.gemstone.WalletConnect
import uniffi.gemstone.WalletConnectAction
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectTransaction
import uniffi.gemstone.WalletConnectTransactionType
import java.math.BigInteger

sealed class WCRequest(
    internal val sessionRequest: Wallet.Model.SessionRequest,
    internal val account: Account,
    private val appMetadata: WalletConnectionSessionAppMetadata,
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
        sessionRequest: Wallet.Model.SessionRequest,
        account: Account,
        appMetadata: WalletConnectionSessionAppMetadata,
        val action: WalletConnectAction.SignMessage,
        val simulation: SimulationResult,
        private val explorerName: String?,
    ) : WCRequest(sessionRequest, account, appMetadata) {

        private val signer by lazy {
            runCatching {
                MessageSigner(walletConnect.decodeSignMessage(action.chain, action.signType, action.data))
            }
        }

        private val payloadPreview by lazy {
            signer.getOrNull()?.let { signer ->
                runCatching { signer.payloadPreview(simulation.payload.map { it.toGem() }) }.getOrNull()
            }
        }

        val plainMessage: String
            get() = signer.getOrNull()?.plainPreview() ?: action.data

        val primaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.primary
                ?.map { it.toPrimitives() }
                .orEmpty()
                .withExplorerLinks(chain, explorerName)
        }

        val secondaryPayloadFields: List<PayloadField> by lazy {
            payloadPreview?.secondary
                ?.map { it.toPrimitives() }
                .orEmpty()
                .withExplorerLinks(chain, explorerName)
        }

        val hasPayload: Boolean
            get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()

        suspend fun execute(privateKey: ByteArray): String {
            val signature = signer.getOrThrow().sign(privateKey)
            return walletConnect.encodeSignMessage(chain.string, signature).payload()
        }
    }

    abstract class Transaction(
        sessionRequest: Wallet.Model.SessionRequest,
        account: Account,
        appMetadata: WalletConnectionSessionAppMetadata,
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
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            appMetadata: WalletConnectionSessionAppMetadata,
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
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            appMetadata: WalletConnectionSessionAppMetadata,
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
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            appMetadata: WalletConnectionSessionAppMetadata,
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
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            appMetadata: WalletConnectionSessionAppMetadata,
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

private fun WalletConnectResponseType.payload(): String = when (this) {
    is WalletConnectResponseType.Object -> json
    is WalletConnectResponseType.String -> value
}

private fun WalletConnectTransaction.map(
    request: WCRequest.Transaction,
    isSendable: Boolean,
): Generic {
    val asset = request.chain.asset()
    return when (this) {
        is WalletConnectTransaction.Ethereum -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            from = request.account,
            memo = data.data,
            name = request.name,
            description = request.description,
            url = request.url,
            icon = request.icon,
            gasLimit = data.gasLimit,
            inputType = request.inputType,
            destination = DestinationAddress(data.to),
            amount = data.value?.hexToBigInteger() ?: BigInteger.ZERO,
            isSendable = isSendable,
        )
        is WalletConnectTransaction.Solana -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            from = request.account,
            memo = data.transaction,
            name = request.name,
            description = request.description,
            url = request.url,
            icon = request.icon,
            gasLimit = "",
            inputType = when (outputType) {
                TransferDataOutputType.ENCODED_TRANSACTION -> ConfirmParams.TransferParams.InputType.EncodeTransaction
                TransferDataOutputType.SIGNATURE -> ConfirmParams.TransferParams.InputType.Signature
            },
            destination = DestinationAddress(""),
            amount = BigInteger.ZERO,
            isSendable = isSendable,
        )
        is WalletConnectTransaction.Sui -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            from = request.account,
            memo = data.transaction,
            name = request.name,
            description = request.description,
            url = request.url,
            icon = request.icon,
            gasLimit = "",
            inputType = when (outputType) {
                TransferDataOutputType.ENCODED_TRANSACTION -> ConfirmParams.TransferParams.InputType.EncodeTransaction
                TransferDataOutputType.SIGNATURE -> ConfirmParams.TransferParams.InputType.Signature
            },
            destination = DestinationAddress(""),
            amount = BigInteger.ZERO,
            isSendable = isSendable,
        )
        is WalletConnectTransaction.Ton -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            from = request.account,
            memo = messages,
            name = request.name,
            description = request.description,
            url = request.url,
            icon = request.icon,
            gasLimit = "",
            inputType = when (outputType) {
                TransferDataOutputType.ENCODED_TRANSACTION -> ConfirmParams.TransferParams.InputType.EncodeTransaction
                TransferDataOutputType.SIGNATURE -> ConfirmParams.TransferParams.InputType.Signature
            },
            destination = DestinationAddress(""),
            amount = BigInteger.ZERO,
            isSendable = isSendable,
        )
        is WalletConnectTransaction.Tron -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            memo = data,
            from = request.account,
            name = request.name,
            description = request.description,
            url = request.url,
            icon = request.icon,
            gasLimit = "",
            inputType = when (outputType) {
                TransferDataOutputType.ENCODED_TRANSACTION -> ConfirmParams.TransferParams.InputType.EncodeTransaction
                TransferDataOutputType.SIGNATURE -> ConfirmParams.TransferParams.InputType.Signature
            },
            destination = DestinationAddress(""),
            amount = BigInteger.ZERO,
            isSendable = isSendable,
        )
    }
}
