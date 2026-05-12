package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.SetCurrentWallet
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull

class SetCurrentWalletImpl(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
) : SetCurrentWallet {

    override suspend fun setCurrentWallet(walletId: WalletId) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        sessionRepository.setWallet(wallet)
    }
}
