package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import dagger.Lazy
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.GemWalletStore

class GemstoneWalletStore(
    private val walletsRepository: Lazy<WalletsRepository>,
) : GemWalletStore {

    override fun getWallets(): List<String> = runBlocking {
        walletsRepository.get().getAll().firstOrNull().orEmpty().map { it.toJson() }
    }

    override fun getWallet(walletId: String): String? = runBlocking {
        walletsRepository.get().getWallet(WalletId(walletId)).firstOrNull()?.toJson()
    }

    override suspend fun addWallet(wallet: String) {
        walletsRepository.get().addWallet(wallet.decodeJson<Wallet>())
    }

    override suspend fun deleteWallet(walletId: String): Boolean = walletsRepository.get().removeWallet(WalletId(walletId))

    override suspend fun setPinned(walletId: String, pinned: Boolean) = updateWallet(walletId) { it.copy(isPinned = pinned) }

    override suspend fun rename(walletId: String, name: String) = updateWallet(walletId) { it.copy(name = name) }

    override suspend fun setImageUrl(walletId: String, imageUrl: String?) = updateWallet(walletId) { it.copy(imageUrl = imageUrl) }

    private suspend fun updateWallet(walletId: String, transform: (Wallet) -> Wallet) {
        val wallet = walletsRepository.get().getWallet(WalletId(walletId)).firstOrNull() ?: return
        walletsRepository.get().updateWallet(transform(wallet))
    }
}
