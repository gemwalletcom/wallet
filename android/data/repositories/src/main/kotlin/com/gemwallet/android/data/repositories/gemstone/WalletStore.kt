package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.WalletId
import dagger.Lazy
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemWalletStore

class GemstoneWalletStore(
    private val walletsRepository: Lazy<WalletsRepository>,
) : GemWalletStore {

    override suspend fun getWallets(): List<String> =
        walletsRepository.get().getAll().firstOrNull().orEmpty().map { it.toJson() }

    override suspend fun getWallet(walletId: String): String? =
        walletsRepository.get().getWallet(WalletId(walletId)).firstOrNull()?.toJson()
}
