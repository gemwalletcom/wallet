package com.gemwallet.android.blockchain.operators

import com.wallet.core.primitives.Wallet

interface DeleteKeyStoreOperator {
    operator fun invoke(wallet: Wallet): Boolean
}
