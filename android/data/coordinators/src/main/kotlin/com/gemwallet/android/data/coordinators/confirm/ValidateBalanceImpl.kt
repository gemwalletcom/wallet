package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.coordinators.ValidateBalance
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
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

        val totalAmount = when (signerParams.input.getTxType()) {
            TransactionType.Transfer,
            TransactionType.Swap,
            TransactionType.TokenApproval,
            TransactionType.AssetActivation,
            TransactionType.StakeFreeze -> amount + if (assetInfo == feeAssetInfo) feeAmount else BigInteger.ZERO
            TransactionType.EarnDeposit,
            TransactionType.StakeDelegate -> if (assetInfo.stakeChain?.freezed() == true) {
                amount
            } else {
                amount + if (assetInfo == feeAssetInfo) feeAmount else BigInteger.ZERO
            }
            TransactionType.StakeUndelegate,
            TransactionType.StakeRewards,
            TransactionType.StakeRedelegate,
            TransactionType.StakeWithdraw,
            TransactionType.EarnWithdraw,
            TransactionType.StakeUnfreeze,
            TransactionType.TransferNFT -> amount
            TransactionType.SmartContractCall,
            TransactionType.PerpetualOpenPosition,
            TransactionType.PerpetualClosePosition,
            TransactionType.PerpetualModifyPosition -> TODO()
        }

        if (assetBalance < totalAmount) {
            val label = "${assetInfo.asset.name} (${assetInfo.asset.symbol})"
            throw ConfirmError.InsufficientBalance(label)
        }
        if (feeAssetInfo.balance.balance.available.toBigInteger() < feeAmount) {
            throw ConfirmError.InsufficientFee(chain = feeAssetInfo.asset.chain)
        }

        val minimumAssetBalance = assetInfo.chain.getMinimumAccountBalance()

        if (!signerParams.input.useMaxAmount
            && !signerParams.input.shouldIgnoreValueCheck
            && assetInfo.asset.type == AssetType.NATIVE
            && minimumAssetBalance > 0L
            && (feeAssetInfo.balance.balance.available.toBigInteger() - totalAmount).let { it > -MAX_256 && it < BigInteger.valueOf(minimumAssetBalance) }) {
            throw ConfirmError.MinimumAccountBalanceTooLow(asset = feeAssetInfo.asset, required = minimumAssetBalance)
        }
    }
}
