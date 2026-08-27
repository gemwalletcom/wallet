package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger

internal fun ConfirmParams.toGemTransactionLoadInput(
    metadata: GemTransactionLoadMetadata,
    finalAmount: BigInteger,
    fee: Fee,
): GemTransactionLoadInput = GemTransactionLoadInput(
    inputType = toDto(),
    senderAddress = from.address,
    destinationAddress = destination()?.address ?: "",
    value = finalAmount.toString(),
    gasPrice = fee.toGemGasPriceType(),
    memo = memo(),
    isMaxValue = useMaxAmount,
    metadata = metadata,
)
