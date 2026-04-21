package com.gemwallet.android.blockchain.operators

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

interface LoadPublicKeyOperator {
    operator fun invoke(wallet: Wallet, chain: Chain): String?
}
