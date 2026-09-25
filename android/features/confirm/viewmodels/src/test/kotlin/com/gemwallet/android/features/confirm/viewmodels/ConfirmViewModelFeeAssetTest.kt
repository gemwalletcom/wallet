package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetEthereumUSDT
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
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemTransactionHeader

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelFeeAssetTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val ethereum = mockAssetEthereum()
    private val usdt = mockAssetEthereumUSDT()
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
    fun changingTheFeeAssetReloadsWithIt() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        viewModel.changeFeeAsset(usdt.id)
        advanceUntilIdle()

        coVerify { confirmation.load(match<GemConfirmLoadOptions> { it.feeAssetId == usdt.id.toIdentifier() }) }
    }

    @Test
    fun reselectingTheLoadedFeeAssetDoesNotReload() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        viewModel.changeFeeAsset(ethereum.id)
        advanceUntilIdle()

        coVerify(exactly = 0) { confirmation.load(match<GemConfirmLoadOptions> { it.feeAssetId != null }) }
    }

    private fun viewModel(): ConfirmViewModel {
        val transfer = mockGemTransferData(asset = ethereum)
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.header(any()) } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(ethereum.toGem()))
        coEvery { confirmation.state() } returns mockGemConfirmLoad(ethereum)
        coEvery { confirmation.load(any()) } answers {
            val options = firstArg<GemConfirmLoadOptions>()
            mockGemConfirmLoad(if (options.feeAssetId == usdt.id.toIdentifier()) usdt else ethereum)
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
