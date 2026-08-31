package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

class GetWalletsImpl(
    private val walletStore: GemstoneWalletStore,
) : GetWallets {
    override fun invoke(): Flow<List<Wallet>> = walletStore.observeWallets()
}

class GetWalletImpl(
    private val walletStore: GemstoneWalletStore,
) : GetWallet {
    override fun invoke(walletId: WalletId): Flow<Wallet?> = walletStore.observeWallet(walletId)
}
