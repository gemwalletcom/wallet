package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.SetWalletAvatar
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.firstOrNull

class SetWalletAvatarImpl(
    private val walletsRepository: WalletsRepository,
) : SetWalletAvatar {

    override suspend fun setWalletAvatar(walletId: WalletId, imageUrl: String?) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        walletsRepository.updateWallet(wallet.copy(imageUrl = imageUrl))
    }
}
