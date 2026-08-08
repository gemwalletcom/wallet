package com.gemwallet.android.domains.transaction.aggregates

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.domains.transaction.values.ValueGroup
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType

interface TransactionDetailsAggregate {
    val id: String
    val asset: Asset
    val type: TransactionType
    val direction: TransactionDirection
    val perpetualDirection: PerpetualDirection?
        get() = null
    val state: TransactionState

    val currency: Currency

    val amount: TransactionDetailsValue.Amount
    val fee: TransactionDetailsValue.Fee
    val date: TransactionDetailsValue.Date
    val status: TransactionDetailsValue.Status
    val rate: TransactionDetailsValue.Rate?
    val swapProgress: TransactionDetailsValue.SwapProgress?
    val swapAgain: TransactionDetailsValue.SwapAgain?
    val memo: TransactionDetailsValue.Memo?
    val resourceType: TransactionDetailsValue.ResourceType?
    val network: TransactionDetailsValue.Network
    val destination: TransactionDetailsValue.Destination?
    val pnl: TransactionDetailsValue.Pnl?
    val price: TransactionDetailsValue.Price?

    val explorer: TransactionDetailsValue.Explorer

    val valueGroups: List<ValueGroup<TransactionDetailsValue>>
}
