package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualId
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
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
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemAutocloseDraft
import uniffi.gemstone.GemAutocloseDraftField
import uniffi.gemstone.GemLeverageSelection
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberFormat
import uniffi.gemstone.GemPerpetualAmountSession
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
    fun `the autoclose sheet checks triggers against the order price`() {
        val provider = makeProvider()
        provider.onAutocloseOpened("1000")

        provider.onAutocloseChanged(TpslType.TakeProfit, "101")
        provider.onAutocloseChanged(TpslType.StopLoss, "99")

        assertEquals(AutocloseValidation.VALID, provider.autocloseViewState.value?.takeProfit?.validation)
        assertEquals(AutocloseValidation.VALID, provider.autocloseViewState.value?.stopLoss?.validation)
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
                asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10u, type = AssetType.TOKEN).toGem(),
                baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8u, type = AssetType.TOKEN).toGem(),
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
        val service = mockk<GemAmountServiceInterface> {
            every { newPerpetualSession(any(), any()) } answers { session(firstArg(), secondArg()) }
        }
        return AmountPerpetualProvider(
            params = AmountParams.Perpetual(assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), perpetualId = mockPerpetualId(symbol = "BTC-PERP"), positionAction = action),
            service = service,
            getCurrentWalletId = getCurrentWalletId,
            assetQuery = assetQuery,
            scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
        )
    }

    private fun session(action: GemPerpetualPositionAction, format: GemNumberFormat): GemPerpetualAmountSession {
        val option = { value: UByte -> GemPickerOption(value = value, label = GemLocalizedText.Text("${value}x")) }
        val empty = GemAutocloseDraftField(value = null, isEdited = false)
        return GemPerpetualAmountSession(
            action = action,
            leverage = GemLeverageSelection(options = listOf(option(5u), option(10u)), selected = option(5u)),
            takeProfitPercent = 0u,
            stopLossPercent = 0u,
            autoclose = GemAutocloseDraft(takeProfit = empty, stopLoss = empty),
            format = format,
        )
    }
}
