package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransferData
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelHeaderTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset()
    private val account = mockAccount(chain = Chain.Bitcoin)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun theHeadShowsWhatCoreComposedWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val viewModel = viewModel(mockGemTransferData(value = BigInteger.valueOf(150_000))).also { model = it }

        val header = viewModel.header.first { it != null }

        assertEquals(asset, (header as ConfirmHeaderUIModel.Symbol).asset)
        assertEquals(FeeUIModel.Calculating, viewModel.feeUIModel.first { it != null })
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        val confirmation = mockk<GemConfirmation> {
            every { rowContents(any()) } returns emptyList()
            every { feeRateRows() } returns null
        }.stubViewState()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header() } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(asset.toGem()))
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset).copy(transfer = transfer)
        coEvery { confirmation.load(any()) } coAnswers { awaitCancellation() }
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
            connectionStatusObserver = mockk(relaxed = true),
            ioDispatcher = testDispatcher,
            context = mockk<Context> {
                every { getString(any()) } returns "Error"
                every { getString(any(), *anyVararg()) } returns "Error"
            },
        )
    }
}
