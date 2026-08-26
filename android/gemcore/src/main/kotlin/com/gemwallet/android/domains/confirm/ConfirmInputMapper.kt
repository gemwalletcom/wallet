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
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransactionInputType

fun ConfirmParams.toConfirmInput(): GemConfirmInput = GemConfirmInput(
    inputType = toDto(),
    from = from.toGem(),
    destination = destination()?.let { GemConfirmDestination(address = it.address, name = it.name) },
    value = amount.toString(),
    memo = memo(),
    references = references,
    useMax = useMaxAmount,
    minimumValue = minimumAmount?.toString(),
)

fun GemConfirmInput.toConfirmParams(): ConfirmParams? {
    val from = from.toPrimitives() ?: return null
    val value = value.toBigIntegerOrNull() ?: return null

    return when (val inputType = inputType) {
        is GemTransactionInputType.Transfer -> {
            val asset = inputType.asset.toPrimitives() ?: return null
            val destination = destination ?: return null
            ConfirmParams.Builder(asset, from, value, useMax)
                .transfer(DestinationAddress(destination.address, destination.name), memo, references)
        }
        is GemTransactionInputType.Generic -> {
            val asset = inputType.asset.toPrimitives() ?: return null
            val extra = inputType.extra
            ConfirmParams.TransferParams.Generic(
                asset = asset,
                from = from,
                amount = value,
                destination = DestinationAddress(destination?.address.orEmpty(), destination?.name),
                memo = memo,
                useMaxAmount = useMax,
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
