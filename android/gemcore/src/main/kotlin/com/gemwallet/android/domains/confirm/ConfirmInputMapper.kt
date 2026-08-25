package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
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
        else -> null
    }
}
