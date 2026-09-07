package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetPrice
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapRate
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.Resource
import java.math.BigInteger

fun mockGemTransactionAmount(
    asset: Asset = mockAsset(),
    value: BigInteger = BigInteger.ONE,
    sign: GemAmountSign = GemAmountSign.NONE,
    price: AssetPrice? = null,
) = GemTransactionAmount(
    asset = asset.toGem(),
    value = value,
    sign = sign,
    price = price?.toGem(),
)

fun mockGemTransactionDetailRows(
    title: GemTransactionTitle = GemTransactionTitle.Sent,
    header: GemTransactionHeader = GemTransactionHeader.Amount(mockGemTransactionAmount(), showsFiat = true),
    headerAction: GemTransactionHeaderAction? = null,
    swapProgress: GemSwapProgress? = null,
    swapAgain: GemSwapAgain? = null,
    estimatedConfirmationSeconds: UInt? = null,
    participant: GemTransactionParticipant? = null,
    providerName: String? = null,
    memo: String? = null,
    resource: Resource? = null,
    rate: GemSwapRate? = null,
    pnl: Double? = null,
    price: Double? = null,
    fee: GemTransactionAmount = mockGemTransactionAmount(),
    explorer: BlockExplorerLink = BlockExplorerLink("Explorer", "https://example.com"),
) = GemTransactionDetailRows(
    title = title,
    header = header,
    headerAction = headerAction,
    swapProgress = swapProgress,
    swapAgain = swapAgain,
    estimatedConfirmationSeconds = estimatedConfirmationSeconds,
    participant = participant,
    providerName = providerName,
    memo = memo,
    resource = resource,
    rate = rate,
    pnl = pnl,
    price = price,
    fee = fee,
    explorer = explorer,
)
