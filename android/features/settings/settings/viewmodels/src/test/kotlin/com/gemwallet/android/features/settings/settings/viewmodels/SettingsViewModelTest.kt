package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
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
    private val walletsRepository = mockk<WalletsRepository>(relaxed = true) {
        every { getAll() } returns wallets
    }
    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { session() } returns MutableStateFlow(null)
    }
    private val switchPushEnabled = mockk<SwitchPushEnabled>(relaxed = true)
    private val getPushEnabled = object : GetPushEnabled {
        override fun getPushEnabled() = MutableStateFlow(true)
    }

    private lateinit var viewModel: SettingsViewModel

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        viewModel = SettingsViewModel(
            userConfig = userConfig,
            walletsRepository = walletsRepository,
            sessionRepository = sessionRepository,
            switchPushEnabled = switchPushEnabled,
            getPushEnabled = getPushEnabled,
            notificationsAvailable = true,
        )
    }

    @After
    fun tearDown() {
        viewModel.viewModelScope.cancel()
        Dispatchers.resetMain()
    }

    @Test
    fun `disableNotifications suppresses the global prompt`() = runTest(testDispatcher) {
        viewModel.disableNotifications()
        advanceUntilIdle()

        coVerify(exactly = 1) { userConfig.stopAskNotifications() }
        coVerify(exactly = 1) { switchPushEnabled.switchPushEnabled(false, emptyList()) }
    }

    @Test
    fun `rewards hidden before wallets load`() = runTest(testDispatcher) {
        assertFalse(viewModel.isRewardsAvailable.value)
    }

    @Test
    fun `single wallet hides rewards`() = runTest(testDispatcher) {
        wallets.value = listOf(mockWallet(type = WalletType.Single))
        advanceUntilIdle()

        assertFalse(viewModel.isRewardsAvailable.value)
    }

    @Test
    fun `multicoin wallet shows rewards`() = runTest(testDispatcher) {
        wallets.value = listOf(mockWallet(type = WalletType.Multicoin))
        advanceUntilIdle()

        assertTrue(viewModel.isRewardsAvailable.value)
    }
}
