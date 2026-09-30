package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemAssetBalance
import com.gemwallet.android.testkit.mockGemConfirmHeader
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.gemwallet.android.testkit.mockGemTransferData
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmTransferViewModelFeeAssetTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val usdt = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7"), name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20)
    private val account = mockAccount(chain = Chain.Ethereum)
    private val confirmation = mockk<GemConfirmation>(relaxed = true).stubViewState()

    @get:Rule
    val confirm = ConfirmTransferRule(testDispatcher, account)

    @Test
    fun changingTheFeeAssetReloadsWithIt() = runTest(testDispatcher) {
        val viewModel = viewModel()
        advanceUntilIdle()

        viewModel.changeFeeAsset(usdt.id)
        advanceUntilIdle()

        coVerify { confirmation.load(match<GemConfirmLoadOptions> { it.feeAssetId == usdt.id.toIdentifier() }) }
    }

    @Test
    fun reselectingTheLoadedFeeAssetDoesNotReload() = runTest(testDispatcher) {
        val viewModel = viewModel()
        advanceUntilIdle()

        viewModel.changeFeeAsset(ethereum.id)
        advanceUntilIdle()

        coVerify(exactly = 0) { confirmation.load(match<GemConfirmLoadOptions> { it.feeAssetId != null }) }
    }

    private fun viewModel(): ConfirmTransferViewModel {
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(ethereum.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE)
        every { confirm.confirmService.confirmation(any(), any(), any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns mockGemConfirmHeader()
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
        return confirm.viewModel(transfer)
    }
}
