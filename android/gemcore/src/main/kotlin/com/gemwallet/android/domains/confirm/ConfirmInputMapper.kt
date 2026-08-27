package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.hex
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.toModel
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import java.nio.ByteBuffer
import java.nio.charset.CharacterCodingException
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData

fun ConfirmParams.toTransferData(): GemTransferData = GemTransferData(
    inputType = toDto(),
    recipient = GemRecipient(
        address = destination()?.address.orEmpty(),
        name = destination()?.name,
        memo = memo(),
        references = references,
    ),
    value = amount.toString(),
    useMaxAmount = useMaxAmount,
    minimumValue = minimumAmount?.toString(),
)

fun ConfirmParams.toConfirmInput(): GemConfirmInput = GemConfirmInput(
    from = from.toGem(),
    transfer = toTransferData(),
)

fun GemConfirmInput.toConfirmParams(): ConfirmParams? {
    val from = from.toPrimitives() ?: return null
    val value = transfer.value.toBigIntegerOrNull() ?: return null
    val recipient = transfer.recipient
    return when (val inputType = transfer.inputType) {
        is GemTransactionInputType.Transfer -> {
            val asset = inputType.asset.toPrimitives() ?: return null
            ConfirmParams.Builder(asset, from, value, transfer.useMaxAmount)
                .transfer(DestinationAddress(recipient.address, recipient.name), recipient.memo, recipient.references)
        }
        is GemTransactionInputType.Generic -> {
            val asset = inputType.asset.toPrimitives() ?: return null
            val extra = inputType.extra
            ConfirmParams.TransferParams.Generic(
                asset = asset,
                from = from,
                amount = value,
                destination = DestinationAddress(recipient.address, recipient.name),
                memo = recipient.memo,
                useMaxAmount = transfer.useMaxAmount,
                inputType = when (extra.outputType.decodeJson<TransferDataOutputType>()) {
                    TransferDataOutputType.Signature -> ConfirmParams.TransferParams.InputType.Signature
                    TransferDataOutputType.EncodedTransaction -> ConfirmParams.TransferParams.InputType.EncodeTransaction
                },
                isSendable = extra.outputAction.decodeJson<TransferDataOutputAction>() == TransferDataOutputAction.Send,
                metadata = inputType.metadata.decodeJson(),
                data = extra.data.toGenericData(),
                gasLimit = extra.gasLimit,
                decodedTransactionType = extra.transactionType.decodeJson(),
                approval = extra.approval?.toModel(),
            )
        }
        else -> null
    }
}

private fun ByteArray?.toGenericData(): String {
    this ?: return ""
    return try {
        Charsets.UTF_8.newDecoder().decode(ByteBuffer.wrap(this)).toString()
    } catch (_: CharacterCodingException) {
        "0x$hex"
    }
}
