package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.SetWalletPinned
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletService

class SetWalletPinnedImpl(
    private val walletService: GemWalletService,
) : SetWalletPinned {

    override suspend fun invoke(walletId: WalletId, pinned: Boolean) {
        walletService.setPinned(walletId.id, pinned)
    }
}
