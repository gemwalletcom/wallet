package com.gemwallet.android.data.coordinators.wallet_import

import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.cases.banners.AddBanner
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletConfigurationService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.WalletConfigurationResult

class SyncWalletConfigurationImpl(
    private val walletConfigurationService: GemWalletConfigurationService,
    private val addBanner: AddBanner,
    private val walletPreferencesFactory: WalletPreferencesFactory,
) : SyncWalletConfiguration {

    override suspend fun sync(walletId: WalletId) {
        val preferences = walletPreferencesFactory.create(walletId.id)
        if (preferences.completeInitialWalletConfiguration) return

        val configuration = runCatching { walletConfigurationService.getConfiguration(walletId.id).decodeJson<WalletConfigurationResult>().configuration }
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
