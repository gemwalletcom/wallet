package com.gemwallet.android.blockchain.operators

import com.wallet.core.primitives.WalletId

interface DeleteKeyStoreOperator {
    operator fun invoke(walletId: WalletId): Boolean
}
