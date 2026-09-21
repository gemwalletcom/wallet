package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionExtended
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapRate
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionFeeRow
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.Resource
import uniffi.gemstone.formattedCurrency
import java.math.BigInteger

fun mockGemTransactionAmount(asset: Asset = mockAsset(), value: BigInteger = BigInteger.ONE, sign: GemAmountSign = GemAmountSign.NONE, price: AssetPrice? = null) = GemTransactionAmount(
    asset = asset.toGem(),
    value = value,
    sign = sign,
    price = price?.toGem(),
)

fun mockGemTransactionDetailRows(
    transaction: TransactionExtended = mockTransactionExtended(),
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
    feeRow: GemTransactionFeeRow = mockGemTransactionFeeRow(fee),
    explorer: BlockExplorerLink = BlockExplorerLink("Explorer", "https://example.com"),
) = GemTransactionDetailRows(
    id = transaction.transaction.id.toIdentifier(),
    asset = transaction.asset.toGem(),
    transactionType = transaction.transaction.type.toGem(),
    direction = transaction.transaction.direction.toGem(),
    state = transaction.transaction.state.toGem(),
    createdAt = transaction.transaction.createdAt,
    title = GemTransactionTitle.Sent,
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
    pnl = pnl?.let { formattedCurrency(it, Currency.USD.string, GemCurrencyStyle.CURRENCY).copy(notation = GemNumberNotation.SIGNED) },
    price = price?.let { formattedCurrency(it, Currency.USD.string, GemCurrencyStyle.CURRENCY) },
    fee = fee,
    feeRow = feeRow,
    explorer = explorer,
    status = GemTransactionStatus(
        tone = GemTransactionStateTone.SUCCESS,
        showsBadge = false,
        showsProgress = false,
    ),
)

fun mockGemTransactionFeeRow(fee: GemTransactionAmount = mockGemTransactionAmount(), fiat: Double? = null) = GemTransactionFeeRow(
    title = GemListRowTitle.NETWORK_FEE,
    amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol(fee.asset.symbol)),
    fiat = fiat?.let { formattedCurrency(it, Currency.USD.string, GemCurrencyStyle.CURRENCY) },
    info = GemInfoTopic.NetworkFee(fee.asset),
)
