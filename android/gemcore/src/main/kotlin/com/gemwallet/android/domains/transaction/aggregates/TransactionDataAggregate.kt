package com.gemwallet.android.domains.transaction.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType

interface TransactionDataAggregate {
    val id: TransactionId
    val asset: Asset
    val address: String
    val addressName: String?
        get() = null
    val value: String
    val equivalentValue: String?
    val type: TransactionType
    val direction: TransactionDirection
    val perpetualDirection: PerpetualDirection?
        get() = null
    val perpetualPrice: Double?
        get() = null
    val pnl: Double?
        get() = null
    val resourceType: Resource?
        get() = null
    val state: TransactionState
    val nftImageUrl: String?
        get() = null

    val isPending: Boolean
        get() = state == TransactionState.Pending

    val createdAt: Long
}
