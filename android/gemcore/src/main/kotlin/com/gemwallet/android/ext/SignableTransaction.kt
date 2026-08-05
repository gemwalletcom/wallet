package com.gemwallet.android.ext

import com.gemwallet.android.math.hexToBigInteger
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionAppMetadata
import com.wallet.core.primitives.TransactionPaymentMetadata
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.SignableTransaction
import uniffi.gemstone.TransferDataOutputType
import java.math.BigInteger

fun SignableTransaction.toConfirmParams(
    requestId: String,
    account: Account,
    appMetadata: TransactionAppMetadata,
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
            appMetadata = appMetadata,
            memo = data.data,
            gasLimit = data.gasLimit,
            inputType = inputType,
            destination = DestinationAddress(data.to),
            amount = data.value?.hexToBigInteger() ?: BigInteger.ZERO,
            isSendable = isSendable,
            transactionType = transactionType.toPrimitives(),
            payment = payment,
        )
        is SignableTransaction.Solana -> encoded(requestId, asset, account, appMetadata, data.transaction, outputType, isSendable, payment)
        is SignableTransaction.Sui -> encoded(requestId, asset, account, appMetadata, data.transaction, outputType, isSendable, payment)
        is SignableTransaction.Ton -> encoded(requestId, asset, account, appMetadata, data, outputType, isSendable, payment)
        is SignableTransaction.Tron -> encoded(requestId, asset, account, appMetadata, data, outputType, isSendable, payment)
    }
}

private fun encoded(
    requestId: String,
    asset: Asset,
    account: Account,
    appMetadata: TransactionAppMetadata,
    payload: String,
    outputType: TransferDataOutputType,
    isSendable: Boolean,
    payment: TransactionPaymentMetadata?,
) = generic(
    requestId = requestId,
    asset = asset,
    account = account,
    appMetadata = appMetadata,
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
    asset: Asset,
    account: Account,
    appMetadata: TransactionAppMetadata,
    memo: String?,
    gasLimit: String?,
    inputType: ConfirmParams.TransferParams.InputType?,
    destination: DestinationAddress,
    amount: BigInteger,
    isSendable: Boolean,
    transactionType: TransactionType = TransactionType.SmartContractCall,
    payment: TransactionPaymentMetadata? = null,
) = ConfirmParams.TransferParams.Generic(
    requestId = requestId,
    asset = asset,
    from = account,
    memo = memo,
    appMetadata = appMetadata,
    gasLimit = gasLimit,
    inputType = inputType,
    destination = destination,
    amount = amount,
    isSendable = isSendable,
    decodedTransactionType = transactionType,
    payment = payment,
)
