package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetRequest
import uniffi.gemstone.GemAcquireAssetFlow
import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
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
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
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
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun networkFeeSheetShowsOncePerErrorAndStaysDismissed() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)
        assertTrue(viewModel.isNetworkFeeSheetVisible.value)

        viewModel.dismissNetworkFeeSheet()
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)
        assertFalse(viewModel.isNetworkFeeSheetVisible.value)

        viewModel.send(FinishConfirmAction { _, _ -> })
        advanceUntilIdle()

        assertTrue(viewModel.isNetworkFeeSheetVisible.value)
    }

    @Test
    fun loadErrorCarriesItsTextAndInfoSheet() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        val error = requireNotNull(viewModel.loadError.value)
        assertEquals("Error", error.text)
        assertTrue(error.info is InfoSheetEntity.NetworkFeeRequiredInfo)

        viewModel.acquire(asset, 10)
        assertEquals(AcquireAssetRequest(asset = asset, buyAmount = 10, offersOptions = false), viewModel.acquireRequest.value)
        viewModel.dismissAcquire()
        assertEquals(null, viewModel.acquireRequest.value)
    }

    private fun viewModel(): ConfirmViewModel {
        val transfer = mockGemTransferData(asset = asset, value = BigInteger.TEN)
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.insufficientNetworkFeeBuyAmount() } returns 10
        every { confirmation.acquireAssetFlow(any()) } returns GemAcquireAssetFlow.FIAT
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
            ioDispatcher = testDispatcher,
            context = mockk<Context> { every { getString(any()) } returns "Error"; every { getString(any(), *anyVararg()) } returns "Error" },
        )
    }
}
