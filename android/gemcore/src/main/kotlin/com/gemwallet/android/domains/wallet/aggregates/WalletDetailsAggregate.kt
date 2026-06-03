package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

interface WalletDetailsAggregate {
    val id: WalletId
    val name: String
    val type: WalletType
    val walletChain: Chain?
    val addresses: List<String>
    val imageUrl: String?
}
