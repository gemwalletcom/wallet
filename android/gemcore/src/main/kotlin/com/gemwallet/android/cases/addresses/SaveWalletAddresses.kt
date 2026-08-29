package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.Wallet

interface SaveWalletAddresses {
    suspend operator fun invoke(wallet: Wallet)
}
