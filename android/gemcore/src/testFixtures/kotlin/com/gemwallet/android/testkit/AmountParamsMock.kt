package com.gemwallet.android.testkit

import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient

fun mockAmountParamsTransfer(assetId: AssetId = mockAssetId(), memo: String? = null) = AmountParams.Transfer(
    assetId = assetId,
    payment = GemPaymentRecipient(GemRecipient(address = "to", memo = memo), null),
)

fun mockAmountParamsPerpetual(positionAction: GemPerpetualPositionAction = GemPerpetualPositionAction.Open(mockGemPerpetualTransferData())) = AmountParams.Perpetual(
    assetId = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"), name = "Bitcoin", symbol = "UBTC", decimals = 10, type = AssetType.TOKEN).id,
    perpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP"),
    positionAction = positionAction,
)
