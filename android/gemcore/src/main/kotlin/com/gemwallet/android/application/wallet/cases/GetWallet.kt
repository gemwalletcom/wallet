package com.gemwallet.android.application.wallet.cases

import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetWallet {
    operator fun invoke(walletId: WalletId): Flow<Wallet?>
}
