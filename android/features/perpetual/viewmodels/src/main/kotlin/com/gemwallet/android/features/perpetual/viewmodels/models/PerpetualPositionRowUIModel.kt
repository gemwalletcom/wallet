package com.gemwallet.android.features.perpetual.viewmodels.models

import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetItemRow

data class PerpetualPositionRowUIModel(val asset: Asset, val row: GemAssetItemRow, val hideBalance: Boolean = false)
