package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetEthereumUSDT
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockPaymentInvoice
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransferDataExtra
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.clearMocks
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
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
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelPaymentAssetTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val ethereum = mockAssetEthereum()
    private val usdt = mockAssetEthereumUSDT()
    private val account = mockAccount(chain = Chain.Ethereum)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>(relaxed = true)
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun paymentAssetIdsAreTheInvoiceQuotes() = runTest(testDispatcher) {
        val viewModel = viewModel(payment(ethereum)).also { model = it }

        assertEquals(listOf(ethereum.id, usdt.id), viewModel.paymentAssetIds.first { it.isNotEmpty() })
    }

    @Test
    fun changingTheAssetReloadsWithItAndTheHeaderFollowsTheLoadedTransfer() = runTest(testDispatcher) {
        val viewModel = viewModel(payment(ethereum)).also { model = it }
        assertEquals(ethereum, viewModel.headerAsset())

        viewModel.changePaymentAsset(usdt.id)
        advanceUntilIdle()

        coVerify { confirmation.load(match<GemConfirmLoadOptions> { it.assetId == usdt.id.toIdentifier() }) }
        assertEquals(usdt, viewModel.header.first { (it as? ConfirmHeaderUIModel.Symbol)?.asset == usdt }.let { (it as ConfirmHeaderUIModel.Symbol).asset })
    }

    @Test
    fun returningToTheSameTransferKeepsTheSelectedPaymentAsset() = runTest(testDispatcher) {
        val transfer = payment(ethereum)
        val viewModel = viewModel(transfer).also { model = it }
        viewModel.headerAsset()
        viewModel.changePaymentAsset(usdt.id)
        advanceUntilIdle()

        clearMocks(confirmation, answers = false)
        viewModel.init(transfer)
        advanceUntilIdle()

        coVerify(exactly = 0) { confirmation.load(any()) }
    }

    @Test
    fun reselectingTheCurrentAssetDoesNotReload() = runTest(testDispatcher) {
        val viewModel = viewModel(payment(ethereum)).also { model = it }
        viewModel.headerAsset()

        viewModel.changePaymentAsset(ethereum.id)
        advanceUntilIdle()

        coVerify(exactly = 0) { confirmation.load(match<GemConfirmLoadOptions> { it.assetId != null }) }
    }

    @Test
    fun changingTheAssetBackWhileTheFirstLoadRunsStillReloads() = runTest(testDispatcher) {
        val gate = CompletableDeferred<Unit>()
        val viewModel = viewModel(payment(ethereum), gate).also { model = it }
        viewModel.headerAsset()

        viewModel.changePaymentAsset(usdt.id)
        advanceUntilIdle()
        viewModel.changePaymentAsset(ethereum.id)
        advanceUntilIdle()
        gate.complete(Unit)
        advanceUntilIdle()

        coVerify { confirmation.load(match<GemConfirmLoadOptions> { it.assetId == ethereum.id.toIdentifier() }) }
    }

    private fun payment(asset: Asset) = mockGemTransferData(
        asset = asset,
        inputType = TransactionInputType.Payment(asset = asset.toGem(), invoice = mockPaymentInvoice(quotes = listOf(ethereum, usdt)), extra = mockTransferDataExtra()),
    )

    private suspend fun ConfirmViewModel.headerAsset() = (header.first { it is ConfirmHeaderUIModel.Symbol } as ConfirmHeaderUIModel.Symbol).asset

    private fun viewModel(transfer: GemTransferData, gate: CompletableDeferred<Unit>? = null): ConfirmViewModel {
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.header(any()) } answers { GemConfirmHeader.Transaction(GemTransactionHeader.Symbol((firstArg<GemConfirmLoad?>()?.transfer ?: transfer).asset.toGem())) }
        coEvery { confirmation.state() } returns mockGemConfirmLoad(ethereum).copy(transfer = transfer)
        coEvery { confirmation.load(any()) } coAnswers {
            val options = firstArg<GemConfirmLoadOptions>()
            if (options.assetId == usdt.id.toIdentifier()) {
                gate?.await()
            }
            val asset = if (options.assetId == usdt.id.toIdentifier()) usdt else ethereum
            mockGemConfirmLoad(asset).copy(transfer = payment(asset))
        }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
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
