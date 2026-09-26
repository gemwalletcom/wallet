package com.gemwallet.android.features.transfer.viewmodels.confirm

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemAssetBalance
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
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmTransferViewModelHeaderTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset()
    private val account = mockAccount(chain = Chain.Bitcoin)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private var model: ConfirmTransferViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun theHeadShowsWhatCoreComposedWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val viewModel = viewModel(mockGemTransferData(inputType = TransactionInputType.Transfer(mockAsset().toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.valueOf(150_000))).also { model = it }

        val header = viewModel.header.first { it != null }

        assertEquals(symbolHeader(asset), header)
        assertEquals(FeeUIModel.Calculating, viewModel.feeUIModel.first { it != null })
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmTransferViewModel {
        val confirmation = mockk<GemConfirmation> {
            every { rowContents(any()) } returns emptyList()
            every { feeRateRows() } returns null
        }.stubViewState()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns symbolHeader(asset)
        coEvery { confirmation.state() } returns
            mockGemConfirmLoad(
                transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                sender = mockAccount(chain = asset.id.chain).toGem(),
                feeAsset = asset.toGem(),
                metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
            ).copy(transfer = transfer)
        coEvery { confirmation.load(any()) } coAnswers { awaitCancellation() }
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        return ConfirmTransferViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
            observeRefreshInterval = mockk(relaxed = true),
            ioDispatcher = testDispatcher,
            context = mockk<Context> {
                every { getString(any()) } returns "Error"
                every { getString(any(), *anyVararg()) } returns "Error"
            },
        )
    }
}
