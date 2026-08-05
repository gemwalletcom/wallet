package com.gemwallet.android.features.payment.viewmodels

import android.util.Log
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.model.Fee
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionPaymentMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import java.math.BigInteger
import javax.inject.Inject

class RecordPayment @Inject constructor(
    private val createTransaction: CreateTransaction,
) {
    suspend fun recordPayment(
        payment: TransactionPaymentMetadata,
        quote: PaymentQuote,
        wallet: Wallet,
    ) {
        val assetId = quote.amount.assetId
        val account = wallet.accounts.firstOrNull { it.chain == assetId.chain }
        if (account == null) {
            Log.e(TAG, "Record payment: no ${assetId.chain} account")
            return
        }
        createTransaction.createTransaction(
            hash = quote.paymentId,
            walletId = wallet.id,
            assetId = assetId,
            owner = account,
            to = "",
            state = TransactionState.Pending,
            fee = Fee.Plain(
                feeAssetId = AssetId(assetId.chain),
                priority = FeePriority.Normal,
                amount = BigInteger.ZERO,
                options = emptyMap(),
            ),
            amount = BigInteger(quote.amount.value),
            memo = "",
            type = TransactionType.Transfer,
            metadata = payment.toJson(),
            direction = TransactionDirection.Outgoing,
            blockNumber = "0",
        )
    }

    private companion object {
        const val TAG = "RecordPayment"
    }
}
