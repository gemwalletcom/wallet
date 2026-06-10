package com.gemwallet.android.model

import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.ext.urlDecode
import com.gemwallet.android.ext.urlEncode
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.util.Base64

@Serializable
sealed interface AmountParams {
    val assetId: AssetId
    val transactionType: TransactionType

    fun pack(): String? = runCatching {
        val json = jsonEncoder.encodeToString(this)
        Base64.getEncoder().encodeToString(json.toByteArray()).urlEncode()
    }.getOrNull()

    @Serializable
    @SerialName("transfer")
    data class Transfer(
        override val assetId: AssetId,
        val destination: DestinationAddress,
        val memo: String? = null,
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
        val positionAction: PerpetualPositionAction,
    ) : AmountParams {
        val direction: PerpetualDirection get() = positionAction.data.direction

        override val transactionType: TransactionType get() = when (positionAction) {
            is PerpetualPositionAction.Open -> TransactionType.PerpetualOpenPosition
            is PerpetualPositionAction.Increase,
            is PerpetualPositionAction.Reduce -> TransactionType.PerpetualModifyPosition
        }
    }

    companion object {
        fun unpack(input: String): AmountParams? = runCatching {
            val json = String(Base64.getDecoder().decode(input.urlDecode()))
            jsonEncoder.decodeFromString<AmountParams>(json)
        }.getOrNull()
    }
}
