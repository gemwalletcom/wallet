package com.gemwallet.android.data.repositories.wallets

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.flow.Flow

interface WalletsRepository {
    suspend fun getNextWalletNumber(): Int

    fun getAll(): Flow<List<Wallet>>

    suspend fun addWatch(walletName: String, address: String, chain: Chain): Wallet

    suspend fun addControlled(
        walletName: String,
        data: String,
        type: WalletType,
        chain: Chain?,
        source: WalletSource
    ): Wallet

    suspend fun updateWallet(wallet: Wallet)

    suspend fun updateAccounts(wallet: Wallet)

    suspend fun removeWallet(walletId: WalletId): Boolean

    fun getWallet(walletId: WalletId): Flow<Wallet?>
}
