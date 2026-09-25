package com.gemwallet.android.features.confirm.viewmodels

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
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetType
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
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelFeeAssetTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val usdt = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7"), name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20)
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
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(ethereum.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE)
        every { confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.header(any()) } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(ethereum.toGem()))
        coEvery { confirmation.state() } returns
            mockGemConfirmLoad(
                transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(ethereum.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                sender = mockAccount(chain = ethereum.id.chain).toGem(),
                feeAsset = ethereum.toGem(),
                metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = ethereum.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = ethereum.id.toIdentifier(), isActive = true)),
                simulation = mockGemConfirmSimulationState(chain = ethereum.id.chain.string),
            )
        coEvery { confirmation.load(any()) } answers {
            val options = firstArg<GemConfirmLoadOptions>()
            (
                if (options.feeAssetId ==
                    usdt.id.toIdentifier()
                ) {
                    usdt
                } else {
                    ethereum
                }
                ).let { asset ->
                mockGemConfirmLoad(
                    transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                    sender = mockAccount(chain = asset.id.chain).toGem(),
                    feeAsset = asset.toGem(),
                    metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                    simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
                )
            }
        }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(account))))
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
