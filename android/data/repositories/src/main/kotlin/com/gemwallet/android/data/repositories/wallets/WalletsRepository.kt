package com.gemwallet.android.data.repositories.wallets

import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface WalletsRepository {

    fun getAll(): Flow<List<Wallet>>



    suspend fun addWallet(wallet: Wallet): Wallet

    suspend fun setPinned(walletId: WalletId, pinned: Boolean)

    suspend fun rename(walletId: WalletId, name: String)

    suspend fun setImageUrl(walletId: WalletId, imageUrl: String?)

    suspend fun updateAccounts(wallet: Wallet)

    suspend fun removeWallet(walletId: WalletId): Boolean

    fun getWallet(walletId: WalletId): Flow<Wallet?>
}
