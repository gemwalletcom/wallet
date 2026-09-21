package com.gemwallet.android.model

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.Resource
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import uniffi.gemstone.Delegation
import uniffi.gemstone.EarnType
import uniffi.gemstone.GemDelegationAmountInput
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemStakeAmountInput

@Serializable
sealed interface AmountParams {
    val assetId: AssetId

    fun pack(): String? = packRoutePayload()

    @Serializable
    @SerialName("transfer")
    data class Transfer(override val assetId: AssetId, val destination: @Contextual GemRecipient, val memo: String? = null, val references: List<String> = emptyList(), val amount: String? = null) : AmountParams

    @Serializable
    @SerialName("perpetual.deposit")
    data class Deposit(override val assetId: AssetId) : AmountParams

    @Serializable
    @SerialName("perpetual.withdraw")
    data class Withdraw(override val assetId: AssetId) : AmountParams

    @Serializable
    sealed interface Stake : AmountParams {

        @Serializable
        @SerialName("stake.delegate")
        data class Delegate(override val assetId: AssetId, val validatorId: String? = null) : Stake

        @Serializable
        @SerialName("stake.undelegate")
        data class Undelegate(override val assetId: AssetId, val validatorId: String, val delegationId: String) : Stake

        @Serializable
        @SerialName("stake.redelegate")
        data class Redelegate(override val assetId: AssetId, val validatorId: String, val delegationId: String) : Stake

        @Serializable
        @SerialName("stake.withdraw")
        data class Withdraw(override val assetId: AssetId, val validatorId: String, val delegationId: String) : Stake

        @Serializable
        @SerialName("stake.rewards")
        data class Rewards(override val assetId: AssetId, val delegations: List<@Contextual Delegation> = emptyList(), val validatorId: String? = null) : Stake

        @Serializable
        @SerialName("stake.freeze")
        data class Freeze(override val assetId: AssetId, val resource: Resource) : Stake

        @Serializable
        @SerialName("stake.unfreeze")
        data class Unfreeze(override val assetId: AssetId, val resource: Resource) : Stake
    }

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
    is GemDelegationAmountInput.Stake -> when (val input = input) {
        is GemStakeAmountInput.Stake -> AmountParams.Stake.Delegate(assetId, validatorId = input.validator?.id)
        is GemStakeAmountInput.Redelegate -> AmountParams.Stake.Redelegate(assetId, input.delegation.validator.id, input.delegation.base.delegationId)
        is GemStakeAmountInput.Unstake -> AmountParams.Stake.Undelegate(assetId, input.delegation.validator.id, input.delegation.base.delegationId)
        is GemStakeAmountInput.Withdraw -> AmountParams.Stake.Withdraw(assetId, input.delegation.validator.id, input.delegation.base.delegationId)
        is GemStakeAmountInput.Rewards -> AmountParams.Stake.Rewards(assetId, input.delegations, input.validator?.id)
        is GemStakeAmountInput.Freeze -> AmountParams.Stake.Freeze(assetId, input.resource.toPrimitives())
        is GemStakeAmountInput.Unfreeze -> AmountParams.Stake.Unfreeze(assetId, input.resource.toPrimitives())
    }

    is GemDelegationAmountInput.Earn -> when (val earnType = earnType) {
        is EarnType.Deposit -> AmountParams.Earn.Deposit(assetId, providerId = earnType.v1.id)
        is EarnType.Withdraw -> AmountParams.Earn.Withdraw(assetId, earnType.v1.validator.id, earnType.v1.base.delegationId)
    }
}
