package com.gemwallet.android.data.coordinators.wallet_import

import com.gemwallet.android.cases.banners.AddBanner
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletConfiguration
import com.wallet.core.primitives.WalletConfigurationResult
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncWalletConfigurationImplTest {

    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val addBanner = mockk<AddBanner>(relaxed = true)
    private val walletPreferences = mockk<WalletPreferences>(relaxed = true) {
        every { completeInitialWalletConfiguration } returns false
    }
    private val walletPreferencesFactory = mockk<WalletPreferencesFactory> {
        every { create(any()) } returns walletPreferences
    }
    private val subject = SyncWalletConfigurationImpl(
        gemDeviceApiClient = gemDeviceApiClient,
        addBanner = addBanner,
        walletPreferencesFactory = walletPreferencesFactory,
    )
    private val walletId = WalletId("wallet-1")

    @Test
    fun sync_addsMultisigBannerForEachReturnedAccountAndMarksComplete() = runTest {
        coEvery { gemDeviceApiClient.getWalletConfiguration("wallet-1") } returns WalletConfigurationResult(
            walletId = walletId,
            configuration = WalletConfiguration(
                multiSignatureAccounts = listOf(
                    ChainAddress(chain = Chain.Tron, address = "tron-address"),
                    ChainAddress(chain = Chain.Solana, address = "sol-address"),
                ),
            ),
        )

        subject.sync(walletId)

        coVerify {
            addBanner.addBanner(
                walletId = walletId,
                asset = null,
                chain = Chain.Tron,
                event = BannerEvent.AccountBlockedMultiSignature,
                state = BannerState.AlwaysActive,
            )
            addBanner.addBanner(
                walletId = walletId,
                asset = null,
                chain = Chain.Solana,
                event = BannerEvent.AccountBlockedMultiSignature,
                state = BannerState.AlwaysActive,
            )
        }
        verify { walletPreferences.completeInitialWalletConfiguration = true }
    }

    @Test
    fun sync_doesNothingWhenMultiSignatureAccountsIsEmpty() = runTest {
        coEvery { gemDeviceApiClient.getWalletConfiguration("wallet-1") } returns WalletConfigurationResult(
            walletId = walletId,
            configuration = WalletConfiguration(multiSignatureAccounts = emptyList()),
        )

        subject.sync(walletId)

        coVerify(exactly = 0) { addBanner.addBanner(any(), any(), any(), any(), any()) }
        verify { walletPreferences.completeInitialWalletConfiguration = true }
    }

    @Test
    fun sync_swallowsApiFailuresAndDoesNotMarkComplete() = runTest {
        coEvery { gemDeviceApiClient.getWalletConfiguration("wallet-1") } throws RuntimeException("network down")

        subject.sync(walletId)

        coVerify(exactly = 0) { addBanner.addBanner(any(), any(), any(), any(), any()) }
        verify(exactly = 0) { walletPreferences.completeInitialWalletConfiguration = true }
    }

    @Test
    fun sync_skipsWhenAlreadyComplete() = runTest {
        every { walletPreferences.completeInitialWalletConfiguration } returns true

        subject.sync(walletId)

        coVerify(exactly = 0) { gemDeviceApiClient.getWalletConfiguration(any()) }
        coVerify(exactly = 0) { addBanner.addBanner(any(), any(), any(), any(), any()) }
    }
}
