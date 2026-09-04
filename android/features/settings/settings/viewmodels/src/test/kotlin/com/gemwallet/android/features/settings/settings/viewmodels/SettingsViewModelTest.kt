package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletSessionServiceInterface
import io.mockk.coVerify
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SettingsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val userConfig = mockk<UserConfig>(relaxed = true)
    private val wallets = MutableStateFlow<List<Wallet>>(emptyList())
    private val session = MutableStateFlow<Session?>(null)
    private val getWallets = mockk<GetWallets>(relaxed = true) {
        every { this@mockk() } returns wallets
    }
    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns session
    }
    private val switchPushEnabled = mockk<SwitchPushEnabled>(relaxed = true)
    private val getPushEnabled = object : GetPushEnabled {
        override fun getPushEnabled() = MutableStateFlow(true)
    }

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        viewModel = createViewModel()
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        viewModel.viewModelScope.coroutineContext.job.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun `disableNotifications suppresses the global prompt`() = runTest(testDispatcher) {
        viewModel.disableNotifications()
        advanceUntilIdle()

        coVerify(exactly = 1) { userConfig.stopAskNotifications() }
        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(false) }
    }

    @Test
    fun `rewards follow core's answer for the loaded wallets`() = runTest(testDispatcher) {
        every { walletSessionService.showsRewards(any()) } returns false
        wallets.value = listOf(mockWallet(type = WalletType.Single))
        viewModel = createViewModel()
        advanceUntilIdle()

        assertFalse(viewModel.isRewardsAvailable.first { !it })

        every { walletSessionService.showsRewards(any()) } returns true
        wallets.value = listOf(mockWallet(type = WalletType.Multicoin))
        advanceUntilIdle()

        assertTrue(viewModel.isRewardsAvailable.first { it })
    }

    private val walletSessionService = mockk<GemWalletSessionServiceInterface>(relaxed = true).also {
        every { it.showsRewards(any()) } returns true
    }

    private val getCurrentCurrency = mockk<GetCurrentCurrency> {
        every { getCurrency() } returns MutableStateFlow(Currency.USD)
    }

    private fun createViewModel() = SettingsViewModel(
        userConfig = userConfig,
        getWallets = getWallets,
        getSession = getSession,
        getCurrentCurrency = getCurrentCurrency,
        switchPushEnabled = switchPushEnabled,
        getPushEnabled = getPushEnabled,
        notificationsAvailable = true,
        walletSessionService = walletSessionService,
    )
}
