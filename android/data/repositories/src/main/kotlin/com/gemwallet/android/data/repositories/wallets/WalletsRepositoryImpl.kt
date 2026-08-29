package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletsRepositoryImpl @Inject constructor(
    private val walletStore: GemstoneWalletStore,
) : WalletsRepository {

    override fun getAll(): Flow<List<Wallet>> = walletStore.observeWallets()

    override suspend fun addWallet(wallet: Wallet): Wallet = walletStore.addWallet(wallet)

    override fun getAllNow(): List<Wallet> = walletStore.getAllNow()

    override fun getWalletNow(walletId: WalletId): Wallet? = walletStore.getWalletNow(walletId)

    override suspend fun setPinned(walletId: WalletId, pinned: Boolean) = walletStore.setPinned(walletId, pinned)

    override suspend fun rename(walletId: WalletId, name: String) = walletStore.rename(walletId, name)

    override suspend fun setImageUrl(walletId: WalletId, imageUrl: String?) = walletStore.setImageUrl(walletId, imageUrl)

    override suspend fun updateAccounts(wallet: Wallet) = walletStore.updateAccounts(wallet)

    override suspend fun removeWallet(walletId: WalletId): Boolean = walletStore.removeWallet(walletId)

    override fun getWallet(walletId: WalletId): Flow<Wallet?> = walletStore.observeWallet(walletId)
}
