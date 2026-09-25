package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemConfirmFee
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.feeAmount
import java.math.BigInteger

fun mockGemConfirmLoad(asset: Asset = mockAssetEthereum(), fee: GemConfirmFee? = null) = GemConfirmLoad(
    transfer = mockGemTransferData(asset = asset),
    sender = mockAccount(chain = asset.id.chain).toGem(),
    feeAsset = asset.toGem(),
    metadata = mockGemConfirmMetadata(asset),
    feeAssets = emptyList(),
    simulation = mockGemConfirmSimulationState(chain = asset.id.chain),
    addressName = null,
    fee = fee,
)

fun mockGemConfirmFee(amount: GemTransferAmountResult = GemTransferAmountResult.Amount(GemTransferAmount(value = BigInteger.ONE, networkFee = BigInteger.ONE, isMaxAmount = false))) = GemConfirmFee(
    value = BigInteger.ONE,
    formatted = feeAmount(mockAssetEthereum().toGem(), BigInteger.ONE, null, Currency.USD.toGem()),
    additionalFees = emptyList(),
    selectedPriority = FeePriority.Normal.toGem(),
    amount = amount,
)
