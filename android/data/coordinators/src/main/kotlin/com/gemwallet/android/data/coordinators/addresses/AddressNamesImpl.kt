package com.gemwallet.android.data.coordinators.addresses

import com.gemwallet.android.application.addresses.cases.RenameWalletAddresses
import com.gemwallet.android.application.addresses.cases.SaveWalletAddresses
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId

class RenameWalletAddressesImpl(
    private val addressStore: GemstoneAddressStore,
) : RenameWalletAddresses {

    override suspend fun rename(walletId: WalletId, name: String) {
        addressStore.renameWalletAddresses(walletId.id, name)
    }
}

class SaveWalletAddressesImpl(
    private val addressStore: GemstoneAddressStore,
) : SaveWalletAddresses {

    override suspend fun invoke(wallet: Wallet) {
        addressStore.saveWalletAddresses(wallet)
    }
}
