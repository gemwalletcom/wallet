package com.gemwallet.android.features.transfer_amount.viewmodels

import uniffi.gemstone.GemRecipient
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountDataProvider
import com.gemwallet.android.features.transfer_amount.viewmodels.providers.AmountProviderFactory
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.ui.models.AmountInputType
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
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
import java.math.BigInteger
import uniffi.gemstone.GemAmountLimits
import uniffi.gemstone.GemAmountService

@OptIn(ExperimentalCoroutinesApi::class)
class AmountViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetCosmos()

    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(
        mockAssetInfo(asset = asset).copy(price = mockAssetPriceInfo(price = 10.0))
    )
    private val availableBalanceFlow = MutableStateFlow(HundredAtom)
    private val limitsFlow = MutableStateFlow<GemAmountLimits?>(null)
    private val reserveForFeeFlow = MutableStateFlow(BigInteger.ZERO)

    private val builtAmounts = mutableListOf<Crypto>()
    private val builtIsMax = mutableListOf<Boolean>()
    private val confirmParams = mockk<ConfirmParams>(relaxed = true)

    private val provider = mockk<AmountDataProvider>(relaxed = true) {
        every { assetInfo } returns assetInfoFlow
        every { availableBalance } returns availableBalanceFlow
        every { minimumValue } returns MutableStateFlow(BigInteger.ZERO)
        every { canChangeValue } returns MutableStateFlow(true)
        every { canSwitchInputType } returns true
        every { reserveForFee } returns reserveForFeeFlow
        every { limits } returns limitsFlow
        every { maxValue() } answers { availableBalanceFlow.value }
        coEvery { buildConfirmParams(capture(builtAmounts), capture(builtIsMax)) } returns confirmParams
    }
    private val factory = mockk<AmountProviderFactory> { every { create(any(), any()) } returns provider }

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `continue is enabled for a valid amount within balance`() = viewModelTest { viewModel ->
        viewModel.setAmount("1")

        assertEquals(ButtonState.Enabled, viewModel.buttonState.value)
        assertTrue(viewModel.amountError.value is AmountError.None)
    }

    @Test
    fun `continue is disabled for empty, zero, and over-balance amounts`() = viewModelTest { viewModel ->
        assertEquals(ButtonState.Disabled, viewModel.buttonState.value)

        viewModel.setAmount("0")
        assertEquals(ButtonState.Disabled, viewModel.buttonState.value)

        availableBalanceFlow.value = OneAtom
        viewModel.setAmount("5")
        assertEquals(ButtonState.Disabled, viewModel.buttonState.value)
        assertTrue(viewModel.amountError.value is AmountError.InsufficientBalance)
    }

    @Test
    fun `onNext converts crypto input to the atomic amount that is sent`() = viewModelTest { viewModel ->
        viewModel.setAmount("1.5")

        assertEquals(confirmParams, viewModel.confirm())
        assertEquals(BigInteger("1500000"), builtAmounts.last().atomicValue)
        assertTrue(viewModel.amountError.value is AmountError.None)
    }

    @Test
    fun `onNext converts fiat input to crypto using the asset price`() = viewModelTest { viewModel ->
        viewModel.switchInputType()
        viewModel.setAmount("20")

        viewModel.confirm()

        assertEquals(BigInteger("2000000"), builtAmounts.last().atomicValue)
    }

    @Test
    fun `onNext rejects an amount over balance without confirming`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = OneAtom
        viewModel.setAmount("5")

        assertNull(viewModel.confirm())
        assertTrue(builtAmounts.isEmpty())
        assertTrue(viewModel.amountError.value is AmountError.InsufficientBalance)
    }

    @Test
    fun `onNext marks isMax when the amount equals the max value`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = OneAtom
        viewModel.setAmount("1")

        viewModel.confirm()

        assertEquals(true, builtIsMax.last())
    }

    @Test
    fun `onNext marks isMax when the amount equals the full balance`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = OneAtom
        viewModel.setAmount("1")

        viewModel.confirm()

        assertEquals(true, builtIsMax.last())
    }

    @Test
    fun `switchInputType flips direction and clears the amount`() = viewModelTest { viewModel ->
        viewModel.setAmount("1")

        viewModel.switchInputType()
        assertEquals(AmountInputType.Fiat, viewModel.amountInputType.value)
        assertEquals("", viewModel.amount)

        viewModel.switchInputType()
        assertEquals(AmountInputType.Crypto, viewModel.amountInputType.value)
    }

    @Test
    fun `onMaxAmount fills the full spendable balance`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = BigInteger("2000000")

        viewModel.onMaxAmount()
        runCurrent()

        assertEquals("2", viewModel.amount)
    }

    @Test
    fun `onMaxAmount reserves the network fee from the balance`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = BigInteger("2000000")
        every { provider.limits } returns MutableStateFlow(GemAmountLimits(availableValue = "2000000", maxValue = "1500000", reservesFee = true))
        every { provider.reserveForFee } returns MutableStateFlow(BigInteger("500000"))
        every { provider.maxValue() } returns BigInteger("1500000")

        viewModel.onMaxAmount()
        runCurrent()

        assertEquals("1.5", viewModel.amount)
    }

    @Test
    fun `typing the max amount by hand shows the reserved fee note`() = viewModelTest { viewModel ->
        availableBalanceFlow.value = BigInteger("2000000")
        limitsFlow.value = GemAmountLimits(availableValue = "2000000", maxValue = "1500000", reservesFee = true)
        reserveForFeeFlow.value = BigInteger("500000")
        every { provider.maxValue() } returns BigInteger("1500000")

        viewModel.setAmount("1")
        assertNull(viewModel.reserveForFeeFormatted.value)

        viewModel.setAmount("1.5")
        assertNotNull(viewModel.reserveForFeeFormatted.value)
    }

    private fun viewModelTest(block: suspend TestScope.(AmountViewModel) -> Unit) = runTest(testDispatcher) {
        val params = AmountParams.Transfer(asset.id, GemRecipient(address = "to", name = null))
        val viewModel = AmountViewModel(factory, SavedStateHandle(mapOf(RouteArgument.Params.key to params.pack())), GemAmountService())
        try {
            runCurrent()
            block(viewModel)
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    private fun AmountViewModel.confirm(): ConfirmParams? {
        var confirmed: ConfirmParams? = null
        onNext { confirmed = it }
        testDispatcher.scheduler.runCurrent()
        return confirmed
    }

    private fun AmountViewModel.setAmount(value: String) {
        updateAmount(value)
        Snapshot.sendApplyNotifications()
        testDispatcher.scheduler.runCurrent()
    }

    private companion object {
        val OneAtom = BigInteger("1000000")
        val HundredAtom = BigInteger("100000000")
    }
}
