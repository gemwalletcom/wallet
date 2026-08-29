package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull

class SetCurrentWalletImpl(
    private val sessionRepository: SessionRepository,
    private val walletStore: GemstoneWalletStore,
) : SetCurrentWallet {

    override suspend fun setCurrentWallet(walletId: WalletId) {
        val wallet = walletStore.observeWallet(walletId).firstOrNull() ?: return
        sessionRepository.setWallet(wallet)
    }
}
