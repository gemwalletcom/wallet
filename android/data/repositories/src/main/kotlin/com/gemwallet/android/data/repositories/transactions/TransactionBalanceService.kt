package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.domains.stake.sumRewardsBalance
import com.gemwallet.android.domains.transaction.TransactionBalanceContext
import com.gemwallet.android.domains.transaction.balance
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.flow.firstOrNull
import java.math.BigInteger
import javax.inject.Inject

class TransactionBalanceService @Inject constructor(
    private val stakeRepository: StakeRepository,
    private val perpetualRepository: PerpetualRepository,
) {

    suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        return assetInfo.balance(params.getTxType(), getContext(assetInfo, params))
    }

    suspend fun getContext(assetInfo: AssetInfo, params: ConfirmParams): TransactionBalanceContext {
        return when (params) {
            is ConfirmParams.Stake.Unfreeze -> TransactionBalanceContext(resource = params.resource)
            is ConfirmParams.Stake.RedelegateParams -> delegationContext(params.delegation)
            is ConfirmParams.Stake.UndelegateParams -> delegationContext(params.delegation)
            is ConfirmParams.Stake.WithdrawParams -> delegationContext(params.delegation)
            is ConfirmParams.Stake.RewardsParams -> TransactionBalanceContext(
                rewardsBalance = getRewardsBalance(assetInfo),
            )
            is ConfirmParams.PerpetualParams.Open -> TransactionBalanceContext(
                perpetualBalance = getPerpetualBalance(assetInfo),
            )
            else -> TransactionBalanceContext()
        }
    }

    private suspend fun delegationContext(delegation: Delegation): TransactionBalanceContext {
        val currentDelegation = stakeRepository.getDelegation(
            validatorId = delegation.base.validatorId,
            delegationId = delegation.base.delegationId,
        ).firstOrNull() ?: delegation
        return TransactionBalanceContext(
            delegationBalance = currentDelegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO,
        )
    }

    suspend fun getBalance(
        assetInfo: AssetInfo,
        params: AmountParams,
        delegation: Delegation? = null,
        resource: Resource? = null,
    ): BigInteger {
        return assetInfo.balance(
            txType = params.txType,
            context = getContext(assetInfo, params, delegation, resource),
        )
    }

    suspend fun getContext(
        assetInfo: AssetInfo,
        params: AmountParams,
        delegation: Delegation? = null,
        resource: Resource? = null,
    ): TransactionBalanceContext {
        return when (params.txType) {
            TransactionType.StakeRewards -> TransactionBalanceContext(
                rewardsBalance = getRewardsBalance(assetInfo),
            )
            TransactionType.StakeUndelegate,
            TransactionType.StakeRedelegate,
            TransactionType.EarnWithdraw,
            TransactionType.StakeWithdraw -> TransactionBalanceContext(
                delegationBalance = delegation?.base?.balance?.toBigIntegerOrNull(),
            )
            TransactionType.StakeUnfreeze -> TransactionBalanceContext(resource = resource)
            else -> TransactionBalanceContext()
        }
    }

    private suspend fun getRewardsBalance(assetInfo: AssetInfo): BigInteger {
        val owner = assetInfo.owner?.address ?: return BigInteger.ZERO
        return stakeRepository.getRewards(assetInfo.asset.id, owner).sumRewardsBalance()
    }

    private suspend fun getPerpetualBalance(assetInfo: AssetInfo): BigInteger {
        val owner = assetInfo.owner?.address ?: return BigInteger.ZERO
        val amount = perpetualRepository.getBalance(owner).firstOrNull()?.available ?: 0.0
        return Crypto(amount.toBigDecimal(), assetInfo.asset.decimals).atomicValue
    }
}
