package com.gemwallet.android.application.wallet.coordinators

import com.wallet.core.primitives.WalletId

interface SetWalletPinned {
    suspend operator fun invoke(walletId: WalletId, pinned: Boolean)
}
