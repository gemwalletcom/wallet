package com.gemwallet.android.ext

import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.TransactionType as GemTransactionType

fun GemTransactionType.toPrimitives(): TransactionType = when (this) {
    GemTransactionType.TRANSFER -> TransactionType.Transfer
    GemTransactionType.TRANSFER_NFT -> TransactionType.TransferNFT
    GemTransactionType.SWAP -> TransactionType.Swap
    GemTransactionType.TOKEN_APPROVAL -> TransactionType.TokenApproval
    GemTransactionType.STAKE_DELEGATE -> TransactionType.StakeDelegate
    GemTransactionType.STAKE_UNDELEGATE -> TransactionType.StakeUndelegate
    GemTransactionType.STAKE_REWARDS -> TransactionType.StakeRewards
    GemTransactionType.STAKE_REDELEGATE -> TransactionType.StakeRedelegate
    GemTransactionType.STAKE_WITHDRAW -> TransactionType.StakeWithdraw
    GemTransactionType.STAKE_FREEZE -> TransactionType.StakeFreeze
    GemTransactionType.STAKE_UNFREEZE -> TransactionType.StakeUnfreeze
    GemTransactionType.ASSET_ACTIVATION -> TransactionType.AssetActivation
    GemTransactionType.SMART_CONTRACT_CALL -> TransactionType.SmartContractCall
    GemTransactionType.PERPETUAL_OPEN_POSITION -> TransactionType.PerpetualOpenPosition
    GemTransactionType.PERPETUAL_CLOSE_POSITION -> TransactionType.PerpetualClosePosition
    GemTransactionType.PERPETUAL_MODIFY_POSITION -> TransactionType.PerpetualModifyPosition
    GemTransactionType.EARN_DEPOSIT -> TransactionType.EarnDeposit
    GemTransactionType.EARN_WITHDRAW -> TransactionType.EarnWithdraw
}
