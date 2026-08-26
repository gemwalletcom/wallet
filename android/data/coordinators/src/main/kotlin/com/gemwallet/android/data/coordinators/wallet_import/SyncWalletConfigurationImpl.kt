package com.gemwallet.android.data.coordinators.wallet_import

import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletConfigurationService

class SyncWalletConfigurationImpl(
    private val walletConfigurationService: GemWalletConfigurationService,
) : SyncWalletConfiguration {

    override suspend fun sync(walletId: WalletId) {
        runCatching { walletConfigurationService.sync(walletId.id) }
    }
}
