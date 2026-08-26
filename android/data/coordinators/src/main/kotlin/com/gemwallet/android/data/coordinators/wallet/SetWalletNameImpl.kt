package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import com.gemwallet.android.cases.addresses.RenameWalletAddresses
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletService

class SetWalletNameImpl(
    private val walletService: GemWalletService,
    private val renameWalletAddresses: RenameWalletAddresses,
) : SetWalletName {

    override suspend fun setWalletName(walletId: WalletId, name: String) {
        walletService.rename(walletId.id, name)
        renameWalletAddresses.rename(walletId, name)
    }
}
