package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualData
import com.gemwallet.android.testkit.mockPerpetualId
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemLeverageSelection
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualAutoclose
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemPickerOption

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProviderTest {

    @Test
    fun `title carries the direction`() {
        val provider = makeProvider(direction = PerpetualDirection.Short)
        val title = provider.request.value?.amountType()?.title() as GemAmountTitle.PerpetualOpen
        assertEquals(PerpetualDirection.Short.toGem(), title.direction)
    }

    @Test
    fun `editing then clearing a trigger keeps it cleared when no default is set`() = runTest {
        val provider = makeProvider(scope = backgroundScope)
        provider.setTakeProfit("65000")
        provider.setStopLoss("55000")
        provider.setTakeProfit("")
        provider.setStopLoss(null)
        advanceUntilIdle()
        assertNull(provider.takeProfit.value)
        assertNull(provider.stopLoss.value)
    }

    @Test
    fun `a leverage change refreshes untouched defaults and keeps an edited price`() = runTest {
        val provider = makeProvider(autoclose = { leverage -> GemPerpetualAutoclose(takeProfit = "${60000 + leverage.toInt()}", stopLoss = "${40000 - leverage.toInt()}") })
        val defaultStopLoss = provider.stopLoss.value

        provider.setTakeProfit("99999")
        provider.setLeverage(10u)

        assertEquals("99999", provider.takeProfit.value)
        assertTrue(provider.stopLoss.value != null && provider.stopLoss.value != defaultStopLoss)
    }

    @Test
    fun `showsAutoclose is true for Open and false for Reduce`() {
        assertTrue(makeProvider().showsAutoclose)
        val reduce =
            makeProvider(
                positionAction = GemPerpetualPositionAction.Reduce(
                    mockGemPerpetualTransferData(
                        provider = PerpetualProvider.Hypercore.toGem(),
                        direction = PerpetualDirection.Long.toGem(),
                        asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10, type = AssetType.TOKEN).toGem(),
                        baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.TOKEN).toGem(),
                        price = 100.0,
                        leverage = 1u,
                        marginType = PerpetualMarginType.Cross.toGem(),
                    ),
                    mockPerpetualPosition().toGem(),
                ),
            )
        assertFalse(reduce.showsAutoclose)
    }

    private fun makeProvider(
        direction: PerpetualDirection = PerpetualDirection.Long,
        positionAction: GemPerpetualPositionAction = GemPerpetualPositionAction.Open(
            mockGemPerpetualTransferData(
                provider = PerpetualProvider.Hypercore.toGem(),
                direction = direction.toGem(),
                asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10, type = AssetType.TOKEN).toGem(),
                baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.TOKEN).toGem(),
                price = 100.0,
                leverage = 1u,
                marginType = PerpetualMarginType.Cross.toGem(),
            ),
        ),
        scope: CoroutineScope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
        autoclose: (UByte) -> GemPerpetualAutoclose = { GemPerpetualAutoclose(takeProfit = null, stopLoss = null) },
    ): AmountPerpetualProvider {
        val walletId = mockWalletId()
        val getCurrentWalletId = mockk<GetCurrentWalletId> {
            every { this@mockk.invoke() } returns flowOf(walletId)
        }
        val assetQuery = mockk<AssetQuery> {
            every { this@mockk.invoke(walletId.id, any()) } returns flowOf(null)
        }
        val service = mockk<GemAmountServiceInterface> {
            every { perpetualLeverageSelection(any()) } returns GemLeverageSelection(
                options = listOf(
                    GemPickerOption(value = 5u, label = GemLocalizedText.Text("5x")),
                    GemPickerOption(value = 10u, label = GemLocalizedText.Text("10x")),
                ),
                selected = GemPickerOption(value = 5u, label = GemLocalizedText.Text("5x")),
            )
            every { perpetualAutoclose(any(), any(), any()) } answers { autoclose(secondArg<Byte>().toUByte()) }
            every { perpetualAutocloseRow(any(), any()) } returns GemListRow.Lines(GemListRowTitle.AUTO_CLOSE, emptyList(), null)
        }
        val perpetualAggregate = mockPerpetualData()
        val perpetualQuery = mockk<PerpetualQuery> {
            every { this@mockk(any<PerpetualId>()) } returns flowOf(perpetualAggregate)
        }
        return AmountPerpetualProvider(
            params = AmountParams.Perpetual(assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), perpetualId = mockPerpetualId(symbol = "BTC-PERP"), positionAction = positionAction),
            context = mockk(relaxed = true),
            service = service,
            getCurrentWalletId = getCurrentWalletId,
            assetQuery = assetQuery,
            perpetualQuery = perpetualQuery,
            scope = scope,
        )
    }
}
