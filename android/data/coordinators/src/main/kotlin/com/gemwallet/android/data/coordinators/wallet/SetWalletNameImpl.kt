package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import kotlinx.coroutines.flow.firstOrNull

class SetWalletNameImpl(
    private val walletsRepository: WalletsRepository,
    private val renameWalletAddresses: RenameWalletAddresses,
) : SetWalletName {

    override suspend fun setWalletName(walletId: String, name: String) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        walletsRepository.updateWallet(wallet.copy(name = name))
        renameWalletAddresses.renameWalletAddresses(walletId, name)
    }
}
