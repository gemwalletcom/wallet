package com.gemwallet.android.model

import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemFeeRate
import java.math.BigInteger

data class SignerParams(
    val input: GemConfirmInput,
    val confirmData: GemConfirmData,
    val fee: Fee,
    val feeRates: List<GemFeeRate>,
    val finalAmount: BigInteger = BigInteger.ZERO,
)
