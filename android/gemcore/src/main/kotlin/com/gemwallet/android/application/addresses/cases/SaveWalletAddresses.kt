package com.gemwallet.android.application.addresses.cases

import com.wallet.core.primitives.Wallet

interface SaveWalletAddresses {
    suspend operator fun invoke(wallet: Wallet)
}
