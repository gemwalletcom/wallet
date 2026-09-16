package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertFalse
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemConfirmTransferService
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelNetworkFeeSheetTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAssetSolana()
    private val account = mockAccount(chain = Chain.Solana)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>()

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun networkFeeSheetShowsOncePerErrorAndStaysDismissed() = runTest(testDispatcher) {
        val viewModel = viewModel()
        runCurrent()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.first { it.phase == GemConfirmPhase.FAILED }.phase)
        assertTrue(viewModel.isNetworkFeeSheetVisible.first { it })

        viewModel.dismissNetworkFeeSheet()
        runCurrent()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)
        assertFalse(viewModel.isNetworkFeeSheetVisible.value)

        viewModel.send(FinishConfirmAction { _ -> })
        runCurrent()

        assertTrue(viewModel.isNetworkFeeSheetVisible.first { it })
    }

    private fun viewModel(): ConfirmViewModel {
        val transfer = mockGemTransferData(asset = asset, value = BigInteger.TEN)
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.insufficientNetworkFeeBuyAmount() } returns 10
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset)
        coEvery { confirmation.load(any()) } answers {
            throw GemConfirmException.InsufficientNetworkFee(asset = asset.toGem(), requirement = null)
        }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            buildConfirmProperties = mockk(relaxed = true),
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
        )
    }
}
