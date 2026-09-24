package com.gemwallet.android.domains.transaction.aggregates

import androidx.compose.runtime.Stable
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemTransactionBadge
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemValueTone

@Stable
interface TransactionDataAggregate {
    val id: TransactionId
    val asset: Asset
    val icon: GemAssetIcon
    val value: String
    val equivalentValue: String?
    val status: GemTransactionStatus
    val title: GemTransactionTitle
    val subtitle: GemTransactionRowSubtitle
    val valueTone: GemValueTone

    val badge: GemTransactionBadge
    val state: TransactionState
    val nftImageUrl: String?
        get() = null

    val createdAt: Long
}
