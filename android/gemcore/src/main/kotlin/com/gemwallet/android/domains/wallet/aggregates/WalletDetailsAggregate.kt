package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

interface WalletDetailsAggregate {
    val id: WalletId
    val name: String
    val type: WalletType
    val addresses: List<String>
}
