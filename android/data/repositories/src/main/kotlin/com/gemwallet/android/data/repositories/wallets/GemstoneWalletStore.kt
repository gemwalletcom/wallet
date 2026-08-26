package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.serializer.toJson
import dagger.Lazy
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemWalletStore

class GemstoneWalletStore(
    private val walletsRepository: Lazy<WalletsRepository>,
) : GemWalletStore {

    override suspend fun getWallets(): List<String> =
        walletsRepository.get().getAll().firstOrNull().orEmpty().map { it.toJson() }
}
