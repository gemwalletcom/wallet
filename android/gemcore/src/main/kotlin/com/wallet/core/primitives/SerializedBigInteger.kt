package com.wallet.core.primitives

import com.gemwallet.android.serializer.BigIntegerSerializer
import kotlinx.serialization.Serializable
import java.math.BigInteger

typealias SerializedBigInteger =
    @Serializable(with = BigIntegerSerializer::class)
    BigInteger
