package com.gemwallet.android.model

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import uniffi.gemstone.EarnType
import uniffi.gemstone.GemDelegationAmountInput
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemStakeAmountInput

@Serializable
sealed interface AmountParams {
    val assetId: AssetId

    fun pack(): String? = packRoutePayload()

    @Serializable
    @SerialName("transfer")
    data class Transfer(override val assetId: AssetId, val payment: @Contextual GemPaymentRecipient) : AmountParams

    @Serializable
    @SerialName("perpetual.deposit")
    data class Deposit(override val assetId: AssetId) : AmountParams

    @Serializable
    @SerialName("perpetual.withdraw")
    data class Withdraw(override val assetId: AssetId) : AmountParams

    @Serializable
    @SerialName("stake")
    data class Stake(override val assetId: AssetId, val input: @Contextual GemStakeAmountInput) : AmountParams

    @Serializable
    sealed interface Earn : AmountParams {

        @Serializable
        @SerialName("earn.deposit")
        data class Deposit(override val assetId: AssetId, val providerId: String) : Earn

        @Serializable
        @SerialName("earn.withdraw")
        data class Withdraw(override val assetId: AssetId, val validatorId: String, val delegationId: String) : Earn
    }

    @Serializable
    @SerialName("perpetual")
    data class Perpetual(override val assetId: AssetId, val perpetualId: PerpetualId, val positionAction: @Contextual GemPerpetualPositionAction) : AmountParams {
        val direction: PerpetualDirection get() = positionAction.transferData().direction.toPrimitives()
    }

    companion object {
        fun unpack(input: String): AmountParams? = unpackRoutePayload(input)
    }
}

fun GemDelegationAmountInput.toAmountParams(assetId: AssetId): AmountParams = when (this) {
    is GemDelegationAmountInput.Stake -> AmountParams.Stake(assetId, input)

    is GemDelegationAmountInput.Earn -> when (val earnType = earnType) {
        is EarnType.Deposit -> AmountParams.Earn.Deposit(assetId, providerId = earnType.v1.id)
        is EarnType.Withdraw -> AmountParams.Earn.Withdraw(assetId, earnType.v1.validator.id, earnType.v1.base.delegationId)
    }
}
