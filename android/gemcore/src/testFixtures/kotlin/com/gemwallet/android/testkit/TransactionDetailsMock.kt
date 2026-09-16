package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionExtended
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemSwapRate
import uniffi.gemstone.formattedCurrency
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemTransactionStatus
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
    transaction: TransactionExtended = mockTransactionExtended(),
    status: GemTransactionStatus = mockGemTransactionStatus(),
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
    currency: Currency = Currency.USD,
    fee: GemTransactionAmount = mockGemTransactionAmount(),
    explorer: BlockExplorerLink = BlockExplorerLink("Explorer", "https://example.com"),
) = GemTransactionDetailRows(
    id = transaction.transaction.id.toIdentifier(),
    asset = transaction.asset.toGem(),
    transactionType = transaction.transaction.type.toGem(),
    direction = transaction.transaction.direction.toGem(),
    state = transaction.transaction.state.toGem(),
    createdAt = transaction.transaction.createdAt,
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
    pnl = pnl?.let { formattedCurrency(it, currency.string, GemCurrencyStyle.CURRENCY).copy(notation = GemNumberNotation.SIGNED) },
    price = price?.let { formattedCurrency(it, currency.string, GemCurrencyStyle.CURRENCY) },
    fee = fee,
    explorer = explorer,
    status = status,
)

fun mockGemTransactionStatus(
    tone: GemTransactionStateTone = GemTransactionStateTone.SUCCESS,
    showsBadge: Boolean = false,
    showsProgress: Boolean = false,
) = GemTransactionStatus(
    tone = tone,
    showsBadge = showsBadge,
    showsProgress = showsProgress,
)
