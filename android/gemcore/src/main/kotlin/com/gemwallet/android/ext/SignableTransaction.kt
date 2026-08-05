package com.gemwallet.android.ext

import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.TransactionPaymentMetadata
import uniffi.gemstone.SignableTransaction
import uniffi.gemstone.TransferDataOutputType
import java.math.BigInteger

data class SigningRequestApp(
    val name: String,
    val description: String,
    val url: String,
    val icon: String,
)

fun SignableTransaction.toConfirmParams(
    requestId: String,
    account: Account,
    app: SigningRequestApp,
    isSendable: Boolean,
    inputType: ConfirmParams.TransferParams.InputType?,
    payment: TransactionPaymentMetadata? = null,
): ConfirmParams.TransferParams.Generic {
    val asset = account.chain.asset()
    return when (this) {
        is SignableTransaction.Ethereum -> generic(
            requestId = requestId,
            asset = asset,
            account = account,
            app = app,
            memo = data.data,
            gasLimit = data.gasLimit,
            inputType = inputType,
            destination = DestinationAddress(data.to),
            amount = data.value?.hexToBigInteger() ?: BigInteger.ZERO,
            isSendable = isSendable,
            transactionType = transactionType.toPrimitives(),
            payment = payment,
        )
        is SignableTransaction.Solana -> encoded(requestId, asset, account, app, data.transaction, outputType, isSendable, payment)
        is SignableTransaction.Sui -> encoded(requestId, asset, account, app, data.transaction, outputType, isSendable, payment)
        is SignableTransaction.Ton -> encoded(requestId, asset, account, app, data, outputType, isSendable, payment)
        is SignableTransaction.Tron -> encoded(requestId, asset, account, app, data, outputType, isSendable, payment)
    }
}

private fun encoded(
    requestId: String,
    asset: com.wallet.core.primitives.Asset,
    account: Account,
    app: SigningRequestApp,
    payload: String,
    outputType: TransferDataOutputType,
    isSendable: Boolean,
    payment: TransactionPaymentMetadata?,
) = generic(
    requestId = requestId,
    asset = asset,
    account = account,
    app = app,
    memo = payload,
    gasLimit = "",
    inputType = when (outputType) {
        TransferDataOutputType.ENCODED_TRANSACTION -> ConfirmParams.TransferParams.InputType.EncodeTransaction
        TransferDataOutputType.SIGNATURE -> ConfirmParams.TransferParams.InputType.Signature
    },
    destination = DestinationAddress(""),
    amount = BigInteger.ZERO,
    isSendable = isSendable,
    payment = payment,
)

private fun generic(
    requestId: String,
    asset: com.wallet.core.primitives.Asset,
    account: Account,
    app: SigningRequestApp,
    memo: String?,
    gasLimit: String?,
    inputType: ConfirmParams.TransferParams.InputType?,
    destination: DestinationAddress,
    amount: BigInteger,
    isSendable: Boolean,
    transactionType: com.wallet.core.primitives.TransactionType = com.wallet.core.primitives.TransactionType.SmartContractCall,
    payment: TransactionPaymentMetadata? = null,
) = ConfirmParams.TransferParams.Generic(
    requestId = requestId,
    asset = asset,
    from = account,
    memo = memo,
    name = app.name,
    description = app.description,
    url = app.url,
    icon = app.icon,
    gasLimit = gasLimit,
    inputType = inputType,
    destination = destination,
    amount = amount,
    isSendable = isSendable,
    decodedTransactionType = transactionType,
    payment = payment,
)
