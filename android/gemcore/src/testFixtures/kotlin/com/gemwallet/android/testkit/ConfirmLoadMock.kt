package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmMetadata
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemFeeAsset
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

fun mockGemConfirmLoad(
    asset: Asset = mockAssetEthereum(),
    metadata: GemConfirmMetadata = mockGemConfirmMetadata(asset),
    feeAssets: List<GemFeeAsset> = emptyList(),
    simulation: GemConfirmSimulationState = GemConfirmSimulationState(
        chain = asset.id.chain.string,
        result = null,
        warnings = emptyList(),
        simulation = null,
        addressNames = emptyList(),
    ),
    addressName: uniffi.gemstone.AddressName? = null,
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
    metadata = metadata,
    feeAssets = feeAssets,
    simulation = simulation,
    addressName = addressName,
    preload = preload,
)
