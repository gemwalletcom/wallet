package com.gemwallet.android.model

import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger

data class SignerParams(
    val input: ConfirmParams,
    val selectedData: Data,
    val feeRates: List<GemFeeRate>,
    val finalAmount: BigInteger = BigInteger.ZERO,
) {
    fun fee(): Fee = selectedData.fee

    fun data(): Data = selectedData

    data class Data(
        val fee: Fee,
        val chainData: ChainSignData,
    )
}

interface ChainSignData {
    fun blockNumber(): String = ""

    fun toDto(): GemTransactionLoadMetadata
}
