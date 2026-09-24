package com.gemwallet.android.testkit

import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.AssetId
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
    assetId = mockAssetHyperCoreUBTC().id,
    perpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP"),
    positionAction = positionAction,
)
