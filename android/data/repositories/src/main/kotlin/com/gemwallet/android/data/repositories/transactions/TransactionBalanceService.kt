package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.domains.confirm.toTransferData
import com.gemwallet.android.domains.stake.rewardsBalance
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
import uniffi.gemstone.GemTransferBalance
import uniffi.gemstone.GemTransferService
import java.math.BigInteger
import javax.inject.Inject

class TransactionBalanceService @Inject constructor(
    private val stakeRepository: StakeRepository,
    private val perpetualRepository: PerpetualRepository,
    private val sessionRepository: SessionRepository,
) {

    suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        if (params is ConfirmParams.PerpetualParams) {
            return getPerpetualBalance(assetInfo)
        }
        val balance = assetInfo.balance.balance
        return GemTransferService().availableValue(
            params.toTransferData(),
            GemTransferBalance(
                available = balance.available,
                frozen = balance.frozen,
                locked = balance.locked,
                withdrawable = balance.withdrawable,
            ),
        ).toBigIntegerOrNull() ?: BigInteger.ZERO
    }

    suspend fun getBalance(
        assetInfo: AssetInfo,
        params: AmountParams,
        delegation: Delegation? = null,
        resource: Resource? = null,
    ): BigInteger {
        return assetInfo.balance(
            transactionType = params.transactionType,
            context = getContext(assetInfo, params, delegation, resource),
        )
    }

    suspend fun getContext(
        assetInfo: AssetInfo,
        params: AmountParams,
        delegation: Delegation? = null,
        resource: Resource? = null,
    ): TransactionBalanceContext {
        return when (params.transactionType) {
            TransactionType.StakeRewards -> TransactionBalanceContext(
                rewardsBalance = delegation?.rewardsBalance() ?: getRewardsBalance(assetInfo),
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
        val walletId = assetInfo.walletId ?: return BigInteger.ZERO
        return stakeRepository.getRewards(walletId, assetInfo.asset.id).sumRewardsBalance()
    }

    private suspend fun getPerpetualBalance(assetInfo: AssetInfo): BigInteger {
        val walletId = sessionRepository.session().value?.wallet?.id ?: return BigInteger.ZERO
        val amount = perpetualRepository.getBalance(walletId, HypercoreUSDC.id).firstOrNull()?.available ?: 0.0
        return Crypto(amount.toBigDecimal(), assetInfo.asset.decimals).atomicValue
    }
}
