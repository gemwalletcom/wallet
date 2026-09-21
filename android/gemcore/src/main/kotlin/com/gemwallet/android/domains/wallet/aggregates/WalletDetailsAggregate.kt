package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSecretKind

interface WalletDetailsAggregate {
    val id: WalletId
    val secretKind: GemWalletSecretKind?
    val row: GemWalletRow
    val address: ChainAddress?
    val addressExplorer: BlockExplorerLink?
}
