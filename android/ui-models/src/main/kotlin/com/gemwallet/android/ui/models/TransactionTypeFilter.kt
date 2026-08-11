package com.gemwallet.android.ui.models

import com.wallet.core.primitives.TransactionType

enum class TransactionTypeFilter {
    Transfer,
    Swap,
    Stake,
    SmartContract,
    Perpetuals,
    Other,
    ;

    val types: List<TransactionType>
        get() = TransactionType.entries.filter { it.filterType == this }
}

val TransactionType.filterType: TransactionTypeFilter
    get() = when (this) {
        TransactionType.Transfer,
        TransactionType.TransferNFT,
        -> TransactionTypeFilter.Transfer

        TransactionType.Swap,
        TransactionType.TokenApproval,
        -> TransactionTypeFilter.Swap

        TransactionType.StakeDelegate,
        TransactionType.StakeUndelegate,
        TransactionType.StakeRewards,
        TransactionType.StakeRedelegate,
        TransactionType.StakeWithdraw,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze,
        TransactionType.EarnDeposit,
        TransactionType.EarnWithdraw,
        -> TransactionTypeFilter.Stake

        TransactionType.SmartContractCall -> TransactionTypeFilter.SmartContract

        TransactionType.PerpetualOpenPosition,
        TransactionType.PerpetualClosePosition,
        TransactionType.PerpetualModifyPosition,
        -> TransactionTypeFilter.Perpetuals

        TransactionType.AssetActivation -> TransactionTypeFilter.Other
    }
