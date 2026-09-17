package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

fun mockGemTransferData(
    asset: Asset = mockAsset(),
    inputType: TransactionInputType = TransactionInputType.Transfer(asset.toGem()),
    recipient: GemRecipient = GemRecipient(address = "recipient"),
    value: BigInteger = BigInteger.ONE,
    useMaxAmount: Boolean = false,
) = GemTransferData(
    inputType = inputType,
    recipient = recipient,
    value = value,
    useMaxAmount = useMaxAmount,
)
