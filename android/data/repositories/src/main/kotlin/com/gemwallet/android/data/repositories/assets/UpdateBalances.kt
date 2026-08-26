package com.gemwallet.android.data.repositories.assets

import uniffi.gemstone.GemBalanceService

class UpdateBalances(
    private val balanceService: GemBalanceService,
) {

    suspend fun updateBalances(walletId: String, assetIds: List<String>) {
        runCatching { balanceService.update(walletId, assetIds) }
    }
}
