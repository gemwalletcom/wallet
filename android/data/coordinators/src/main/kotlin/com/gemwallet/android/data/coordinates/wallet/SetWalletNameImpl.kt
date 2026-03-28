package com.gemwallet.android.data.coordinates.wallet

import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import com.gemwallet.android.data.repositoreis.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AddressesDao
import kotlinx.coroutines.flow.firstOrNull

class SetWalletNameImpl(
    private val walletsRepository: WalletsRepository,
    private val addressDao: AddressesDao,
) : SetWalletName {

    override suspend fun setWalletName(walletId: String, name: String) {
        val wallet = walletsRepository.getWallet(walletId).firstOrNull() ?: return
        walletsRepository.updateWallet(wallet.copy(name = name))
        addressDao.updateNameByWalletId(walletId, name)
    }
}