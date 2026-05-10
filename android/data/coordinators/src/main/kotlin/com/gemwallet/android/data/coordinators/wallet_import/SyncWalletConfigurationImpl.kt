package com.gemwallet.android.data.coordinators.wallet_import

import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.cases.banners.AddBanner
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.WalletId

class SyncWalletConfigurationImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val addBanner: AddBanner,
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : SyncWalletConfiguration {

    override suspend fun sync(walletId: WalletId) {
        val preferences = walletPreferencesFactory.create(walletId.id)
        if (preferences.completeInitialWalletConfiguration) return

        val configuration = runCatching { gemDeviceApiClient.getWalletConfiguration(walletId.id).configuration }
            .getOrNull() ?: return

        configuration.multiSignatureAccounts.forEach { account ->
            addBanner.addBanner(
                walletId = walletId,
                chain = account.chain,
                event = BannerEvent.AccountBlockedMultiSignature,
                state = BannerState.AlwaysActive,
            )
        }
        preferences.completeInitialWalletConfiguration = true
    }
}
