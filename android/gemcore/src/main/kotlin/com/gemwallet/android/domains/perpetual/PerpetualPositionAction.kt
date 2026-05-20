package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.serializer.BigIntegerSerializer
import com.wallet.core.primitives.PerpetualDirection
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.math.BigInteger

@Serializable
sealed interface PerpetualPositionAction {
    val data: PerpetualTransferData

    @Serializable
    @SerialName("open")
    data class Open(override val data: PerpetualTransferData) : PerpetualPositionAction

    @Serializable
    @SerialName("increase")
    data class Increase(override val data: PerpetualTransferData) : PerpetualPositionAction

    @Serializable
    @SerialName("reduce")
    data class Reduce(
        override val data: PerpetualTransferData,
        @Serializable(BigIntegerSerializer::class)
        val available: BigInteger,
        val positionDirection: PerpetualDirection,
    ) : PerpetualPositionAction
}
