package com.gemwallet.features.bridge.viewmodels.model

import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.ConfirmParams.TransferParams.Generic
import com.gemwallet.android.model.DestinationAddress
import com.reown.walletkit.client.Wallet
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import uniffi.gemstone.MessageSigner
import uniffi.gemstone.TransferDataOutputType
import uniffi.gemstone.WalletConnect
import uniffi.gemstone.WalletConnectAction
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectTransaction
import uniffi.gemstone.WalletConnectTransactionType
import uniffi.gemstone.WalletConnectionVerificationStatus
import java.math.BigInteger

sealed class WCRequest(
    internal val sessionRequest: Wallet.Model.SessionRequest,
    internal val account: Account,
    val verificationStatus: WalletConnectionVerificationStatus,
) {
    internal val walletConnect = WalletConnect()

    val requestId: Long get() = sessionRequest.request.id

    val topic: String get() = sessionRequest.topic

    val name: String get() = sessionRequest.peerMetaData?.name ?: ""

    val icon: String get() = sessionRequest.peerMetaData?.icons?.firstOrNull() ?: ""
    val description: String get() = sessionRequest.peerMetaData?.description ?: ""
    val uri: String get() = sessionRequest.peerMetaData?.url?.getShortUrl() ?: ""
    val method: String get() = sessionRequest.request.method

    val chain: Chain get() = account.chain

    class SignMessage(
        sessionRequest: Wallet.Model.SessionRequest,
        account: Account,
        verificationStatus: WalletConnectionVerificationStatus,
        val action: WalletConnectAction.SignMessage,
    ) : WCRequest(sessionRequest, account, verificationStatus) {

        val signer: MessageSigner
            get() = MessageSigner(walletConnect.decodeSignMessage(action.chain, action.signType, action.data))

        suspend fun execute(privateKey: ByteArray): String {
            val signature = signer.sign(privateKey)
            return walletConnect.encodeSignMessage(chain.string, signature).payload()
        }
    }

    abstract class Transaction(
        sessionRequest: Wallet.Model.SessionRequest,
        account: Account,
        verificationStatus: WalletConnectionVerificationStatus,
        val isSendable: Boolean,
        val inputType: ConfirmParams.TransferParams.InputType,
        val transactionType: WalletConnectTransactionType,
        val data: String,
    ) : WCRequest(sessionRequest, account, verificationStatus) {

        open val confirmParams: Generic
            get() = walletConnect.decodeSendTransaction(transactionType, data).map(this, isSendable)

        abstract fun execute(result: String): String

        abstract class Signing(
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            verificationStatus: WalletConnectionVerificationStatus,
            transactionType: WalletConnectTransactionType,
            data: String,
        ) : Transaction(
            sessionRequest = sessionRequest,
            account = account,
            verificationStatus = verificationStatus,
            isSendable = false,
            inputType = ConfirmParams.TransferParams.InputType.Signature,
            transactionType = transactionType,
            data = data,
        )

        class SignTransaction(
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            verificationStatus: WalletConnectionVerificationStatus,
            val action: WalletConnectAction.SignTransaction,
        ) : Signing(
            sessionRequest = sessionRequest,
            account = account,
            verificationStatus = verificationStatus,
            transactionType = action.transactionType,
            data = action.data,
        ) {

            override fun execute(result: String): String =
                walletConnect.encodeSignTransaction(action.chain, result).payload()
        }

        class SignAllTransactions(
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            verificationStatus: WalletConnectionVerificationStatus,
            val action: WalletConnectAction.SignAllTransactions,
        ) : Signing(
            sessionRequest = sessionRequest,
            account = account,
            verificationStatus = verificationStatus,
            transactionType = action.transactionType,
            data = action.transactions.singleOrNull()
                ?: throw BridgeRequestError.MethodUnsupported,
        ) {

            override fun execute(result: String): String =
                walletConnect.encodeSignAllTransactions(listOf(result)).payload()
        }

        class SendTransaction(
            sessionRequest: Wallet.Model.SessionRequest,
            account: Account,
            verificationStatus: WalletConnectionVerificationStatus,
            val action: WalletConnectAction.SendTransaction,
        ) : Transaction(
            sessionRequest = sessionRequest,
            account = account,
            verificationStatus = verificationStatus,
            isSendable = true,
            inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
            transactionType = action.transactionType,
            data = action.data,
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
            url = request.uri,
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
            url = request.uri,
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
            url = request.uri,
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
        is WalletConnectTransaction.Bitcoin -> Generic(
            requestId = request.requestId.toString(),
            asset = asset,
            from = request.account,
            name = request.name,
            description = request.description,
            url = request.uri,
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
            name = request.name,
            description = request.description,
            url = request.uri,
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
            url = request.uri,
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
