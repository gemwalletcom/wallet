package com.gemwallet.android.domains.transaction

import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.getDelegatePreparedAmount
import com.gemwallet.android.model.rewardsBalance
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import java.math.BigInteger

data class TransactionBalanceContext(
    val delegationBalance: BigInteger? = null,
    val rewardsBalance: BigInteger? = null,
    val resource: Resource? = null,
    val perpetualBalance: BigInteger? = null,
)

fun AssetInfo.balance(
    txType: TransactionType,
    context: TransactionBalanceContext = TransactionBalanceContext(),
): BigInteger {
    return when (txType) {
        TransactionType.Transfer,
        TransactionType.Swap,
        TransactionType.TokenApproval,
        TransactionType.StakeFreeze,
        TransactionType.AssetActivation,
        TransactionType.TransferNFT,
        TransactionType.SmartContractCall -> balance.balance.available.toBigInteger()

        TransactionType.EarnDeposit,
        TransactionType.StakeDelegate -> if (stakeChain?.freezed() == true) {
            balance.balance.getDelegatePreparedAmount()
        } else {
            balance.balance.available.toBigInteger()
        }

        TransactionType.StakeRewards -> context.rewardsBalance ?: balance.balance.rewardsBalance()

        TransactionType.StakeUndelegate,
        TransactionType.StakeRedelegate,
        TransactionType.EarnWithdraw,
        TransactionType.StakeWithdraw -> context.delegationBalance ?: BigInteger.ZERO

        TransactionType.StakeUnfreeze -> when (context.resource) {
            Resource.Energy -> balance.balance.locked.toBigInteger()
            Resource.Bandwidth, null -> balance.balance.frozen.toBigInteger()
        }

        TransactionType.PerpetualOpenPosition -> context.perpetualBalance ?: BigInteger.ZERO
        TransactionType.PerpetualClosePosition,
        TransactionType.PerpetualModifyPosition -> throw IllegalArgumentException()
    }
}
