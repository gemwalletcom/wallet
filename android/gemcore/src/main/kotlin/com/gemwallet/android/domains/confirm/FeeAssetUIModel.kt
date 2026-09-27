package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemFeeAsset

data class FeeAssetUIModel(val asset: Asset, val row: GemAssetItemRow)

fun GemFeeAsset.toFeeAssetUIModel(): FeeAssetUIModel = FeeAssetUIModel(asset.toPrimitives(), row)
