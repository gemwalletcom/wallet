package com.gemwallet.android.features.transfer.viewmodels.confirm

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemAssetBalance
import com.gemwallet.android.testkit.mockGemAssetIcon
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.WalletId
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
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmTransferViewModelRequestTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val account = mockAccount(chain = Chain.Ethereum)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>(relaxed = true).stubViewState()
    private var model: ConfirmTransferViewModel? = null

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
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = account.address, memo = "m".repeat(64 * 1024)), value = BigInteger.ONE)
        val handle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack())))
        model = viewModel(handle)
        advanceUntilIdle()

        verify(exactly = 1) { confirmService.confirmation(any(), transfer, any()) }
    }

    @Test
    fun theLatestParamsReplaceTheEarlierOnes() = runTest(testDispatcher) {
        val first = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = account.address, memo = "first"), value = BigInteger.ONE)
        val second = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = account.address, memo = "second"), value = BigInteger.ONE)
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
        val connected = mockWallet(id = WalletId("wallet-2"), name = "Connected", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xconnected")))
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = account.address, memo = "connected"), value = BigInteger.ONE)
        val viewModel = viewModel(SavedStateHandle()).also { model = it }

        viewModel.init(transfer, wallet = connected)
        advanceUntilIdle()

        verify(exactly = 1) { confirmService.confirmation(connected.toGem(), transfer, any()) }
        verify(exactly = 0) { confirmService.confirmation(current.toGem(), any(), any()) }
    }

    @Test
    fun initForTheSameTransferKeepsTheChosenFee() = runTest(testDispatcher) {
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = account.address, memo = "fee"), value = BigInteger.ONE)
        val viewModel = viewModel(SavedStateHandle()).also { model = it }
        viewModel.init(transfer)
        advanceUntilIdle()
        viewModel.changeFeePriority(FeePriority.Fast)
        advanceUntilIdle()

        viewModel.init(transfer)
        advanceUntilIdle()

        assertEquals(FeePriority.Fast, viewModel.feeSelectionUIModel.value.selectedPriority)
    }

    private fun viewModel(handle: SavedStateHandle): ConfirmTransferViewModel {
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(asset.toGem(), mockGemAssetIcon()))
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.errorInfo(any()) } returns null
        coEvery { confirmation.state() } returns
            mockGemConfirmLoad(
                transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                sender = mockAccount(chain = asset.id.chain).toGem(),
                feeAsset = asset.toGem(),
                metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
            )
        coEvery { confirmation.load(any()) } throws IllegalStateException("preload failed")
        return confirmTransferViewModel(handle)
    }

    private fun confirmTransferViewModel(handle: SavedStateHandle) = ConfirmTransferViewModel(
        getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
        },
        confirmService = confirmService,
        savedStateHandle = handle,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
    )
}
