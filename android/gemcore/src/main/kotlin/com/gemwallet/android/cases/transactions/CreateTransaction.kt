package com.gemwallet.android.cases.transactions

import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import java.math.BigInteger

interface CreateTransaction {
    suspend fun createTransaction(
        hash: String,
        walletId: WalletId,
        assetId: AssetId,
        owner: Account,
        to: String,
        state: TransactionState,
        fee: Fee,
        amount: BigInteger,
        memo: String?,
        type: TransactionType,
        metadata: String? = null,
        direction: TransactionDirection,
        blockNumber: String,
    ): Transaction
}
