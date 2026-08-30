package com.gemwallet.android.ext

import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.FeeConfig

fun feeRateDecimals(feeUnitType: FeeUnitType?, feeConfig: FeeConfig, assetDecimals: Int): Int =
    when (feeUnitType) {
        FeeUnitType.SatVb, FeeUnitType.Gwei -> feeConfig.decimals.toInt()
        FeeUnitType.Native, null -> assetDecimals
    }
