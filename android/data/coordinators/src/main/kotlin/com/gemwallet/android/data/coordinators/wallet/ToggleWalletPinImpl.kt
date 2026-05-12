package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.ToggleWalletPin
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull

class ToggleWalletPinImpl(
    private val walletsRepository: WalletsRepository,
) : ToggleWalletPin {

    override suspend fun toggleWalletPin(walletId: WalletId) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        walletsRepository.updateWallet(wallet.copy(isPinned = !wallet.isPinned))
    }
}
