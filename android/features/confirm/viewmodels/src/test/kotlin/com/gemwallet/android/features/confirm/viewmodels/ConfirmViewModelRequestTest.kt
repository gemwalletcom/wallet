package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemButtonState
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionHeader

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelRequestTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAssetEthereum()
    private val account = mockAccount(chain = Chain.Ethereum)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>(relaxed = true).stubViewState()
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun malformedParamsNeverBecomeARequest() = runTest(testDispatcher) {
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to "not a packed payload"))
        val viewModel = viewModel(handle).also { model = it }

        assertNull(viewModel.title.first())
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)
        assertEquals(GemButtonState.LOADING, viewModel.button.value.state)
        verify(exactly = 0) { confirmService.confirmation(any(), any(), any()) }
    }

    @Test
    fun aLargePayloadIsDecodedIntoTheRequest() = runTest(testDispatcher) {
        val transfer = mockGemTransferData(asset = asset, recipient = GemRecipient(address = account.address, memo = "m".repeat(64 * 1024)))
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack())))
        model = viewModel(handle)
        advanceUntilIdle()

        verify(exactly = 1) { confirmService.confirmation(any(), transfer, any()) }
    }

    @Test
    fun theLatestParamsReplaceTheEarlierOnes() = runTest(testDispatcher) {
        val first = mockGemTransferData(asset = asset, recipient = GemRecipient(address = account.address, memo = "first"))
        val second = mockGemTransferData(asset = asset, recipient = GemRecipient(address = account.address, memo = "second"))
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(first.pack())))
        model = viewModel(handle)
        advanceUntilIdle()
        verify { confirmService.confirmation(any(), first, any()) }

        handle[RouteArgument.Params.key] = requireNotNull(second.pack())
        advanceUntilIdle()

        verify { confirmService.confirmation(any(), second, any()) }
    }

    @Test
    fun aRequestForAnotherWalletIsConfirmedForThatWallet() = runTest(testDispatcher) {
        val current = mockWallet(accounts = listOf(account))
        val connected = mockWallet(id = "wallet-2", name = "Connected", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xconnected")))
        val transfer = mockGemTransferData(asset = asset, recipient = GemRecipient(address = account.address, memo = "connected"))
        val viewModel = viewModel(SavedStateHandle()).also { model = it }

        viewModel.init(transfer, wallet = connected)
        advanceUntilIdle()

        verify(exactly = 1) { confirmService.confirmation(connected.toGem(), transfer, any()) }
        verify(exactly = 0) { confirmService.confirmation(current.toGem(), any(), any()) }
    }

    @Test
    fun initForTheSameTransferKeepsTheChosenFee() = runTest(testDispatcher) {
        val transfer = mockGemTransferData(asset = asset, recipient = GemRecipient(address = account.address, memo = "fee"))
        val viewModel = viewModel(SavedStateHandle()).also { model = it }
        viewModel.init(transfer)
        advanceUntilIdle()
        viewModel.changeFeePriority(FeePriority.Fast)
        advanceUntilIdle()

        viewModel.init(transfer)
        advanceUntilIdle()

        assertEquals(FeePriority.Fast, viewModel.feeSelectionUIModel.value.selectedPriority)
    }

    private fun viewModel(handle: SavedStateHandle): ConfirmViewModel {
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(asset.toGem()))
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.errorInfo(any()) } returns null
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset)
        coEvery { confirmation.load(any()) } throws IllegalStateException("preload failed")
        return confirmViewModel(handle)
    }

    private fun confirmViewModel(handle: SavedStateHandle) = ConfirmViewModel(
        getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
        },
        confirmService = confirmService,
        savedStateHandle = handle,
        connectionStatusObserver = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
    )
}
