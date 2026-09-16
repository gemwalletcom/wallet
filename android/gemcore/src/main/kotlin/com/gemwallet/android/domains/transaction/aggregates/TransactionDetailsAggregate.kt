package com.gemwallet.android.domains.transaction.aggregates

import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailSection
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionTitle

interface TransactionDetailsAggregate {
    val id: String
    val asset: Asset
    val title: GemTransactionTitle

    val type: TransactionType
    val direction: TransactionDirection
    val state: TransactionState

    val currency: Currency

    val headerAction: GemTransactionHeaderAction?
    val fee: TransactionDetailsValue.Fee
    val explorer: TransactionDetailsValue.Explorer

    val sections: List<GemTransactionDetailSection>

    fun value(row: GemTransactionDetailRow): TransactionDetailsValue
}
