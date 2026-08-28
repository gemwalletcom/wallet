package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import dagger.Lazy
import uniffi.gemstone.GemWalletStore

class GemstoneWalletStore(
    private val walletsRepository: Lazy<WalletsRepository>,
) : GemWalletStore {

    override fun getWallets(): List<String> = walletsRepository.get().getAllNow().map { it.toJson() }

    override fun getWallet(walletId: String): String? = walletsRepository.get().getWalletNow(WalletId(walletId))?.toJson()

    override suspend fun addWallet(wallet: String) {
        walletsRepository.get().addWallet(wallet.decodeJson<Wallet>())
    }

    override suspend fun deleteWallet(walletId: String): Boolean = walletsRepository.get().removeWallet(WalletId(walletId))

    override suspend fun setPinned(walletId: String, pinned: Boolean) = walletsRepository.get().setPinned(WalletId(walletId), pinned)

    override suspend fun setName(walletId: String, name: String) = walletsRepository.get().rename(WalletId(walletId), name)

    override suspend fun setImageUrl(walletId: String, imageUrl: String?) = walletsRepository.get().setImageUrl(WalletId(walletId), imageUrl)
}
