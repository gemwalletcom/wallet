package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.cases.SetWalletPinned
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SetWalletPinnedImpl(
    private val walletService: GemWalletService,
) : SetWalletPinned {

    override suspend fun invoke(walletId: WalletId, pinned: Boolean) = withContext(Dispatchers.IO) {
        walletService.setPinned(walletId.id, pinned)
    }
}
