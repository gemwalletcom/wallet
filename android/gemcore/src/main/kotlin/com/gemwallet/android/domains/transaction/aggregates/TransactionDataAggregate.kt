package com.gemwallet.android.domains.transaction.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionTitle

interface TransactionDataAggregate {
    val id: TransactionId
    val asset: Asset
    val address: String
    val value: String
    val equivalentValue: String?
    val title: GemTransactionTitle
    val subtitle: GemTransactionRowSubtitle
    val valueSign: GemAmountSign

    val type: TransactionType
    val direction: TransactionDirection
    val pnl: Double?
        get() = null
    val state: TransactionState
    val nftImageUrl: String?
        get() = null

    val isPending: Boolean
        get() = state == TransactionState.Pending

    val createdAt: Long
}
