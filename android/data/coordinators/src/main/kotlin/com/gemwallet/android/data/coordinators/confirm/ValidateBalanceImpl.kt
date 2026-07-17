package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.coordinators.ValidateBalance
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.domains.confirm.BalanceRequirement
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.ext.getMinimumAccountBalance
import com.gemwallet.android.math.MAX_256
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.TransactionType
import java.math.BigInteger

class ValidateBalanceImpl : ValidateBalance {

    override fun invoke(
        signerParams: SignerParams,
        assetInfo: AssetInfo,
        feeAssetInfo: AssetInfo,
        assetBalance: BigInteger,
    ) {
        val amount = signerParams.finalAmount
        val feeAmount = signerParams.fee().amount
        val feeBalance = feeAssetInfo.balance.balance.available.toBigInteger()
        val amountWithFee = amount + if (assetInfo == feeAssetInfo) feeAmount else BigInteger.ZERO

        val totalAmount = when (signerParams.input.getTransactionType()) {
            TransactionType.Transfer,
            TransactionType.Swap,
            TransactionType.TokenApproval,
            TransactionType.AssetActivation,
            TransactionType.StakeFreeze -> amountWithFee
            TransactionType.EarnDeposit,
            TransactionType.StakeDelegate -> if (assetInfo.stakeChain?.freezed() == true) {
                amount
            } else {
                amountWithFee
            }
            TransactionType.StakeUndelegate,
            TransactionType.StakeRewards,
            TransactionType.StakeRedelegate,
            TransactionType.StakeWithdraw,
            TransactionType.EarnWithdraw,
            TransactionType.StakeUnfreeze,
            TransactionType.TransferNFT,
            TransactionType.PerpetualOpenPosition,
            TransactionType.PerpetualClosePosition,
            TransactionType.PerpetualModifyPosition,
            TransactionType.SmartContractCall -> amount
        }

        if (!signerParams.input.shouldIgnoreValueCheck && assetBalance < totalAmount) {
            throw ConfirmError.InsufficientBalance(
                asset = assetInfo.asset,
                requirement = BalanceRequirement(required = totalAmount, available = assetBalance),
            )
        }
        val minimumAmount = signerParams.input.minimumAmount
        if (minimumAmount != null && amount < minimumAmount) {
            throw ConfirmError.InsufficientBalance(
                asset = assetInfo.asset,
                requirement = BalanceRequirement(required = minimumAmount, available = amount),
            )
        }
        if (feeBalance < feeAmount) {
            throw ConfirmError.InsufficientFee(
                chain = feeAssetInfo.asset.chain,
                requirement = BalanceRequirement(
                    required = feeAmount,
                    available = feeBalance,
                ),
            )
        }

        val minimumAccountBalance = BigInteger.valueOf(assetInfo.chain.getMinimumAccountBalance())
        val remainingBalance = feeBalance - totalAmount

        if (!signerParams.input.useMaxAmount
            && !signerParams.input.shouldIgnoreValueCheck
            && assetInfo.asset.type == AssetType.NATIVE
            && minimumAccountBalance > BigInteger.ZERO
            && remainingBalance > -MAX_256
            && remainingBalance < minimumAccountBalance) {
            throw ConfirmError.MinimumAccountBalanceTooLow(
                asset = feeAssetInfo.asset,
                requirement = BalanceRequirement(
                    required = minimumAccountBalance,
                    available = remainingBalance,
                ),
            )
        }
    }
}
