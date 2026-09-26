package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualId
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction

class AmountParamsPerpetualTest {

    @Test
    fun direction_derivesFromPositionActionData() {
        val data =
            mockGemPerpetualTransferData(
                provider = PerpetualProvider.Hypercore.toGem(),
                direction = PerpetualDirection.Short.toGem(),
                asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10, type = AssetType.TOKEN).toGem(),
                baseAsset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.TOKEN).toGem(),
                price = 100.0,
                leverage = 1u,
                marginType = PerpetualMarginType.Cross.toGem(),
            )
        assertEquals(
            PerpetualDirection.Short,
            AmountParams.Perpetual(
                assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"),
                perpetualId = mockPerpetualId(symbol = "BTC-PERP"),
                positionAction = GemPerpetualPositionAction.Open(data),
            ).direction,
        )
    }

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = AmountParams.Perpetual(
            assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"),
            perpetualId = mockPerpetualId(symbol = "BTC-PERP"),
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
                mockPerpetualPosition(marginAmount = 1.5).toGem(),
            ),
        )

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}
