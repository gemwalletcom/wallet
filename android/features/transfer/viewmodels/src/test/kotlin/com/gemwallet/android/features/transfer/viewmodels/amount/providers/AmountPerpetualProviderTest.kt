package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.testkit.mockGemPerpetualAmountSession
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockHyperCoreBtc
import com.gemwallet.android.testkit.mockHyperCoreUsdc
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.TpslType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemLeverageSelection
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemPickerOption

class AmountPerpetualProviderTest {

    @Test
    fun `title carries the direction`() {
        val provider = makeProvider(direction = PerpetualDirection.Short)
        val title = provider.request.value?.amountType()?.title() as GemAmountTitle.PerpetualOpen
        assertEquals(PerpetualDirection.Short.toGem(), title.direction)
    }

    @Test
    fun `a leverage pick reaches the request and keeps the submitted trigger`() {
        val provider = makeProvider()
        provider.onAutocloseOpened("1000")
        provider.onAutocloseChanged(TpslType.TakeProfit, "120")
        assertTrue(provider.onAutocloseSubmitted())

        provider.setLeverage(10u)

        assertEquals(10.toUByte(), (provider.request.value?.amountType() as GemAmountType.Perpetual).leverage)
        provider.onAutocloseOpened("1000")
        assertEquals("120", provider.autocloseViewState.value?.takeProfit?.text)
    }

    @Test
    fun `triggers on the wrong side of the order price are not submitted`() {
        val provider = makeProvider()
        provider.onAutocloseOpened("1000")
        provider.onAutocloseChanged(TpslType.TakeProfit, "99")
        provider.onAutocloseChanged(TpslType.StopLoss, "101")

        assertFalse(provider.onAutocloseSubmitted())

        provider.onAutocloseOpened("1000")
        assertEquals("", provider.autocloseViewState.value?.takeProfit?.text)
        assertEquals("", provider.autocloseViewState.value?.stopLoss?.text)
    }

    @Test
    fun `a trigger cleared in the sheet stays cleared`() {
        val provider = makeProvider()
        provider.onAutocloseOpened("1000")
        provider.onAutocloseChanged(TpslType.TakeProfit, "120")
        provider.onAutocloseChanged(TpslType.StopLoss, "90")
        assertTrue(provider.onAutocloseSubmitted())

        provider.onAutocloseOpened("1000")
        provider.onAutocloseChanged(TpslType.TakeProfit, "")
        assertTrue(provider.onAutocloseSubmitted())
        provider.onAutocloseOpened("1000")

        assertEquals("", provider.autocloseViewState.value?.takeProfit?.text)
        assertEquals("90", provider.autocloseViewState.value?.stopLoss?.text)
    }

    private fun makeProvider(direction: PerpetualDirection = PerpetualDirection.Long): AmountPerpetualProvider {
        val action = GemPerpetualPositionAction.Open(
            mockGemPerpetualTransferData(
                provider = PerpetualProvider.Hypercore.toGem(),
                direction = direction.toGem(),
                asset = mockHyperCoreBtc().toGem(),
                baseAsset = mockHyperCoreUsdc().toGem(),
                price = 100.0,
                leverage = 10u,
                marginType = PerpetualMarginType.Cross.toGem(),
            ),
        )
        val walletId = mockWalletId()
        val getCurrentWalletId = mockk<GetCurrentWalletId> {
            every { this@mockk.invoke() } returns flowOf(walletId)
        }
        val assetQuery = mockk<AssetQuery> {
            every { this@mockk.invoke(walletId.id, any()) } returns flowOf(null)
        }
        val option = { value: UByte -> GemPickerOption(value = value, label = GemLocalizedText.Text("${value}x")) }
        return AmountPerpetualProvider(
            initialSession = mockGemPerpetualAmountSession(
                action = action,
                leverage = GemLeverageSelection(options = listOf(option(5u), option(10u)), selected = option(5u)),
                format = numberFormat(),
            ),
            getCurrentWalletId = getCurrentWalletId,
            assetQuery = assetQuery,
            scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
        )
    }
}
