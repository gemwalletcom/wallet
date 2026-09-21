package com.gemwallet.android.features.referral.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockGemRewardsState
import com.gemwallet.android.testkit.mockRewards
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemIncomingCode
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.Rewards

@OptIn(ExperimentalCoroutinesApi::class)
class ReferralViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val wallet = mockWalletMulticoin(address = "0xabc")
    private val secondWallet = mockWalletMulticoin(address = "0xdef", name = "Second Wallet")
    private val walletsFlow = MutableStateFlow(listOf(wallet))
    private val sessionFlow = MutableStateFlow<Session?>(mockSession(wallet))

    private val getWallets = object : GetWallets {
        override fun invoke() = walletsFlow
    }
    private val getSession = object : GetSession {
        override fun invoke(): StateFlow<Session?> = sessionFlow
    }
    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }
    private val service = mockk<GemRewardsServiceInterface> {
        every { wallets(any()) } answers { firstArg() }
        every { selectedWallet(any(), any()) } answers { firstArg() }
        every { state(any()) } answers { mockGemRewardsState(referralCode = firstArg<Rewards?>()?.code, usedReferralCode = firstArg<Rewards?>()?.usedReferralCode) }
        coEvery { getRewards(any()) } returns mockRewards()
        coEvery { useReferralCode(any(), any()) } returns mockRewards(usedReferralCode = "friend")
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `using a code shows the state it produced`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            runCurrent()
            assertNull(viewModel.uiState.value.usedReferralCode)

            viewModel.useCode("friend") {}
            runCurrent()

            assertEquals("friend", viewModel.uiState.value.usedReferralCode)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `an incoming code is activated with one wallet and confirmed with more`() = runTest(testDispatcher) {
        val viewModel = createViewModel(code = "friend")

        try {
            runCurrent()
            assertEquals(GemIncomingCode.Activate("friend"), viewModel.incomingCode.value)

            walletsFlow.value = listOf(wallet, secondWallet)
            runCurrent()

            assertEquals(GemIncomingCode.Confirm("friend"), viewModel.incomingCode.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `no incoming code decides nothing`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            runCurrent()

            assertNull(viewModel.incomingCode.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun createViewModel(code: String? = null): ReferralViewModel {
        val arguments = mutableMapOf<String, Any>()
        code?.let { arguments[RouteArgument.Code.key] = it }
        return ReferralViewModel(
            getSession = getSession,
            getWallets = getWallets,
            service = service,
            savedStateHandle = SavedStateHandle(arguments),
            context = context,
            ioDispatcher = testDispatcher,
        )
    }
}
