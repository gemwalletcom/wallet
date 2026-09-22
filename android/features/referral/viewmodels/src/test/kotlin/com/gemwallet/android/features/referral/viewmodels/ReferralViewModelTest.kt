package com.gemwallet.android.features.referral.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.features.referral.viewmodels.models.IncomingCodeUIModel
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockGemRewardsResult
import com.gemwallet.android.testkit.mockRewards
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWalletMulticoin
import com.gemwallet.android.ui.R
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemRewardsAction
import uniffi.gemstone.GemRewardsServiceInterface
import uniffi.gemstone.GemServiceException

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
        coEvery { refresh(any()) } answers { mockGemRewardsResult(walletId = firstArg()) }
        coEvery { useReferralCode(any(), any()) } returns mockRewards(usedReferralCode = "friend", verifyAfter = 4_102_444_800)
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
            assertNull(pendingCode(viewModel))

            viewModel.useCode("friend") {}
            runCurrent()

            assertEquals("friend", pendingCode(viewModel))
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed load reads as an error instead of a wallet without a code`() = runTest(testDispatcher) {
        coEvery { service.refresh(any()) } answers {
            mockGemRewardsResult(walletId = firstArg(), rewards = null, error = GemServiceException.Gateway("offline"))
        }
        val viewModel = createViewModel()

        try {
            runCurrent()

            assertEquals("offline", (viewModel.loadError.value as? GemServiceException.Gateway)?.msg)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a load for a wallet that is no longer shown is dropped`() = runTest(testDispatcher) {
        coEvery { service.refresh(any()) } answers {
            mockGemRewardsResult(walletId = wallet.id.id, rewards = mockRewards(code = "first"))
        }
        walletsFlow.value = listOf(wallet, secondWallet)
        val viewModel = createViewModel()

        try {
            runCurrent()
            assertEquals("https://gemwallet.com/join?code=first", viewModel.referralLink.value)

            viewModel.setWallet(secondWallet.id.id)
            runCurrent()

            assertNull("the first wallet's code must not follow the selection", viewModel.referralLink.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `an incoming code is activated with one wallet and confirmed with more`() = runTest(testDispatcher) {
        val viewModel = createViewModel(code = "friend")

        try {
            runCurrent()
            assertEquals(IncomingCodeUIModel(activate = "friend"), viewModel.incomingCode.value)

            walletsFlow.value = listOf(wallet, secondWallet)
            runCurrent()

            assertEquals(IncomingCodeUIModel(confirm = "friend"), viewModel.incomingCode.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `no incoming code decides nothing`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        try {
            runCurrent()

            assertEquals(IncomingCodeUIModel(), viewModel.incomingCode.value)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `the info section shows the code, the referral count, the points and the inviter`() = runTest(testDispatcher) {
        coEvery { service.refresh(any()) } answers {
            mockGemRewardsResult(walletId = firstArg(), rewards = mockRewards(code = "GEM123", usedReferralCode = "FRIEND", points = 250))
        }
        val viewModel = createViewModel()

        try {
            runCurrent()

            val rows = viewModel.sections.value.flatMap { it.rows }
            assertEquals(
                listOf(R.string.rewards_my_referral_code, R.string.rewards_referrals, R.string.rewards_points, R.string.rewards_invited_by).map { "string:$it" },
                rows.map { it.title },
            )
            assertEquals("GEM123", rows[0].subtitle)
            assertTrue(rows[2].subtitle.orEmpty().contains("250"))
            assertEquals("FRIEND", rows[3].subtitle)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun pendingCode(viewModel: ReferralViewModel): String? = viewModel.actions.value.filterIsInstance<GemRewardsAction.ActivatePendingReferral>().firstOrNull()?.code

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
