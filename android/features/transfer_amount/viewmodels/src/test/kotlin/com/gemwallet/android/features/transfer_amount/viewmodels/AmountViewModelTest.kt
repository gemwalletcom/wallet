package com.gemwallet.android.features.transfer_amount.viewmodels

import android.content.Context
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBalance
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockBalance
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockGemValidatorRow
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.DelegationValidator
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAmountInputType
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeAmountSelection
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemTransferData
import java.math.BigDecimal
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6)

    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(assetInfo(HundredAtom))

    private val sentValues = mutableListOf<BigInteger>()
    private val sentIsMax = mutableListOf<Boolean>()
    private val confirmInput = mockk<GemTransferData>(relaxed = true)

    private val service = mockk<GemAmountServiceInterface>(relaxed = true) {
        every { getCurrency() } returns Currency.USD.toGem()
        coEvery { transferData(any(), any(), capture(sentValues), capture(sentIsMax)) } returns confirmInput
    }
    private val stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
        every { stakeAmountSelection(any(), any()) } returns
            GemStakeAmountSelection.Validator(
                mockDelegationValidator(chain = Chain.Cosmos).let { validator ->
                    mockGemValidatorRow(validator = validator.toGem(), name = validator.name, placeholder = validator.name.take(1), apr = GemLocalizedText.Apr(null))
                },
                true,
            )
    }
    private val walletId = mockWalletId()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> { every { this@mockk.invoke() } returns flowOf(walletId) }
    private val assetQuery = mockk<AssetQuery> { every { this@mockk.invoke(walletId.id, any()) } returns assetInfoFlow }

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }

    @Test
    fun `continue is enabled for a valid amount within balance`() = viewModelTest { viewModel ->
        viewModel.setAmount("1")

        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertEquals("", viewModel.uiState.value.error)
    }

    @Test
    fun `continue is disabled for empty, zero, and over-balance amounts`() = viewModelTest { viewModel ->
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        viewModel.setAmount("0")
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        assetInfoFlow.value = assetInfo(OneAtom)
        viewModel.setAmount("5")
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)
        assertEquals("string:${R.string.transfer_insufficient_balance}", viewModel.uiState.value.error)
    }

    @Test
    fun `a prefilled payment amount fills the amount once`() = viewModelTest(AmountParams.Transfer(asset.id, GemPaymentRecipient(GemRecipient(address = "to"), "1.5"))) { viewModel ->
        assertEquals("1.5", viewModel.amount)

        viewModel.setAmount("")
        assetInfoFlow.value = assetInfo(OneAtom)
        runCurrent()
        assertEquals("", viewModel.amount)
    }

    @Test
    fun `onNext sends the atomic amount the crypto input names`() = viewModelTest { viewModel ->
        viewModel.setAmount("1.5")

        assertEquals(confirmInput, viewModel.confirm())
        assertEquals(BigInteger("1500000"), sentValues.last())
        assertEquals("", viewModel.uiState.value.error)
    }

    @Test
    fun `onNext converts fiat input to crypto using the asset price`() = viewModelTest { viewModel ->
        viewModel.switchInputType()
        viewModel.setAmount("20")

        viewModel.confirm()

        assertEquals(BigInteger("2000000"), sentValues.last())
    }

    @Test
    fun `onNext rejects an amount over balance without confirming`() = viewModelTest { viewModel ->
        assetInfoFlow.value = assetInfo(OneAtom)
        viewModel.setAmount("5")

        assertNull(viewModel.confirm())
        assertTrue(sentValues.isEmpty())
        assertEquals("string:${R.string.transfer_insufficient_balance}", viewModel.uiState.value.error)
    }

    @Test
    fun `onNext marks isMax when the amount equals the max value`() = viewModelTest { viewModel ->
        assetInfoFlow.value = assetInfo(OneAtom)
        viewModel.setAmount("1")

        viewModel.confirm()

        assertEquals(true, sentIsMax.last())
    }

    @Test
    fun `switchInputType flips direction and clears the amount`() = viewModelTest { viewModel ->
        viewModel.setAmount("1")

        viewModel.switchInputType()
        assertEquals(GemAmountInputType.FIAT, viewModel.amountInputType.value)
        assertEquals("", viewModel.amount)

        viewModel.switchInputType()
        assertEquals(GemAmountInputType.ASSET, viewModel.amountInputType.value)
    }

    @Test
    fun `onMaxAmount in fiat mode enters the max in asset units`() = viewModelTest { viewModel ->
        assetInfoFlow.value = assetInfo(BigInteger("2000000"))
        runCurrent()
        viewModel.switchInputType()

        viewModel.onMaxAmount()
        Snapshot.sendApplyNotifications()
        runCurrent()

        assertEquals(GemAmountInputType.ASSET, viewModel.amountInputType.value)
        assertEquals("2", viewModel.amount)
        viewModel.confirm()
        assertEquals(BigInteger("2000000"), sentValues.last())
        assertEquals(true, sentIsMax.last())
    }

    @Test
    fun `onMaxAmount fills the full spendable balance`() = viewModelTest { viewModel ->
        assetInfoFlow.value = assetInfo(BigInteger("2000000"))
        runCurrent()

        viewModel.onMaxAmount()
        runCurrent()

        assertEquals("2", viewModel.amount)
    }

    @Test
    fun `a stake max keeps the network fee back and says so`() = viewModelTest(AmountParams.Stake(asset.id, GemStakeAmountInput.Stake(mockDelegationValidator(chain = Chain.Cosmos).toGem()))) { viewModel ->
        assetInfoFlow.value = assetInfo(BigInteger("2000000"))
        runCurrent()

        viewModel.onMaxAmount()
        runCurrent()
        Snapshot.sendApplyNotifications()
        testDispatcher.scheduler.advanceUntilIdle()

        assertTrue("a stake max stays under the balance", viewModel.amount.toBigDecimal() < BigDecimal(2))
        assertNotNull(viewModel.uiState.value.reserveForFee)
    }

    private fun assetInfo(available: BigInteger) =
        mockAssetInfo(asset = asset, balance = mockAssetBalance(asset = asset, balance = mockBalance(available = available)), price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 10.0)))

    private fun viewModelTest(params: AmountParams = AmountParams.Transfer(assetId = asset.id, payment = GemPaymentRecipient(GemRecipient(address = "to", memo = null), null)), block: suspend TestScope.(AmountViewModel) -> Unit) =
        runTest(testDispatcher) {
            val viewModel = AmountViewModel(
                service = service,
                stakeService = stakeService,
                getCurrentWalletId = getCurrentWalletId,
                assetQuery = assetQuery,
                perpetualQuery = mockk(relaxed = true),
                delegationQuery = mockk(relaxed = true),
                validatorQuery = mockk(relaxed = true),
                validatorsQuery = mockk { every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList<DelegationValidator>()) },
                getSession = mockk(relaxed = true),
                savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to params.pack())),
                context = context,
                ioDispatcher = testDispatcher,
            )
            try {
                runCurrent()
                block(viewModel)
            } finally {
                viewModel.viewModelScope.cancel()
            }
        }

    private fun AmountViewModel.confirm(): GemTransferData? {
        var confirmed: GemTransferData? = null
        onNext { confirmed = it.data }
        testDispatcher.scheduler.runCurrent()
        return confirmed
    }

    private fun AmountViewModel.setAmount(value: String) {
        updateAmount(value)
        Snapshot.sendApplyNotifications()
        testDispatcher.scheduler.advanceUntilIdle()
    }

    private companion object {
        val OneAtom = BigInteger("1000000")
        val HundredAtom = BigInteger("100000000")
    }
}
