package com.gemwallet.android.domains.transaction.aggregates

import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType

interface TransactionDataAggregate {
    val id: String
    val asset: Asset
    val address: String
    val addressName: String?
        get() = null
    val addressType: AddressType?
        get() = null
    val value: String
    val equivalentValue: String?
    val type: TransactionType
    val direction: TransactionDirection
    val state: TransactionState
    val nftImageUrl: String?
        get() = null

    val isPending: Boolean
        get() = state == TransactionState.Pending

    val createdAt: Long
}