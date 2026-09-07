package com.gemwallet.android.model

import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemRecipient
import com.gemwallet.android.domains.perpetual.data
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemPerpetualPositionAction
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.Contextual

@Serializable
sealed interface AmountParams {
    val assetId: AssetId
    val transactionType: TransactionType
    val amount: String? get() = null

    fun pack(): String? = packRoutePayload()

    @Serializable
    @SerialName("transfer")
    data class Transfer(
        override val assetId: AssetId,
        val destination: @Contextual GemRecipient,
        val memo: String? = null,
        val references: List<String> = emptyList(),
        override val amount: String? = null,
    ) : AmountParams {
        override val transactionType: TransactionType get() = TransactionType.Transfer
    }

    @Serializable
    @SerialName("perpetual.deposit")
    data class Deposit(
        override val assetId: AssetId,
    ) : AmountParams {
        override val transactionType: TransactionType get() = TransactionType.Transfer
    }

    @Serializable
    @SerialName("perpetual.withdraw")
    data class Withdraw(
        override val assetId: AssetId,
    ) : AmountParams {
        override val transactionType: TransactionType get() = TransactionType.Transfer
    }

    @Serializable
    sealed interface Stake : AmountParams {

        @Serializable @SerialName("stake.delegate")
        data class Delegate(
            override val assetId: AssetId,
            val validatorId: String? = null,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeDelegate
        }

        @Serializable @SerialName("stake.undelegate")
        data class Undelegate(
            override val assetId: AssetId,
            val validatorId: String,
            val delegationId: String,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeUndelegate
        }

        @Serializable @SerialName("stake.redelegate")
        data class Redelegate(
            override val assetId: AssetId,
            val validatorId: String,
            val delegationId: String,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeRedelegate
        }

        @Serializable @SerialName("stake.withdraw")
        data class Withdraw(
            override val assetId: AssetId,
            val validatorId: String,
            val delegationId: String,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeWithdraw
        }

        @Serializable @SerialName("stake.rewards")
        data class Rewards(
            override val assetId: AssetId,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeRewards
        }

        @Serializable @SerialName("stake.freeze")
        data class Freeze(
            override val assetId: AssetId,
            val resource: Resource,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeFreeze
        }

        @Serializable @SerialName("stake.unfreeze")
        data class Unfreeze(
            override val assetId: AssetId,
            val resource: Resource,
        ) : Stake {
            override val transactionType: TransactionType get() = TransactionType.StakeUnfreeze
        }
    }

    @Serializable
    @SerialName("perpetual")
    data class Perpetual(
        override val assetId: AssetId,
        val perpetualId: PerpetualId,
        val positionAction: @Contextual GemPerpetualPositionAction,
    ) : AmountParams {
        val direction: PerpetualDirection get() = positionAction.transferData().direction.toPrimitives()

        override val transactionType: TransactionType get() = when (positionAction) {
            is GemPerpetualPositionAction.Open -> TransactionType.PerpetualOpenPosition
            is GemPerpetualPositionAction.Increase,
            is GemPerpetualPositionAction.Reduce -> TransactionType.PerpetualModifyPosition
        }
    }

    companion object {
        fun unpack(input: String): AmountParams? = unpackRoutePayload(input)
    }
}
