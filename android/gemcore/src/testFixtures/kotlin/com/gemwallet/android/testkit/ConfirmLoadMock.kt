package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

fun mockGemConfirmLoad(
    asset: Asset = mockAssetEthereum(),
    preload: GemConfirmPreload? = null,
) = GemConfirmLoad(
    transfer = GemTransferData(
        inputType = TransactionInputType.Transfer(asset.toGem()),
        recipient = GemRecipient(address = "", name = null, memo = null, references = emptyList()),
        value = BigInteger.ZERO,
        useMaxAmount = false,
    ),
    sender = mockAccount(chain = asset.id.chain).toGem(),
    feeAsset = asset.toGem(),
    metadata = mockGemConfirmMetadata(asset),
    feeAssets = emptyList(),
    simulation = mockGemConfirmSimulationState(chain = asset.id.chain),
    addressName = null,
    preload = preload,
)
