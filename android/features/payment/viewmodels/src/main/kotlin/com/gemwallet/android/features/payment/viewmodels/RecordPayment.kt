package com.gemwallet.android.features.payment.viewmodels

import android.util.Log
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.Fee
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionPaymentMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import java.math.BigInteger
import javax.inject.Inject

class RecordPayment @Inject constructor(
    private val createTransaction: CreateTransaction,
) {
    suspend operator fun invoke(
        provider: PaymentProviderName,
        quotes: GemPaymentQuotes,
        quote: GemPaymentQuote,
        wallet: Wallet,
    ) {
        val assetId = quote.amount.assetId.toAssetId()
        if (assetId == null) {
            Log.e(TAG, "Record payment: bad asset ${quote.amount.assetId}")
            return
        }
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
            metadata = TransactionPaymentMetadata(
                paymentId = quote.paymentId,
                merchant = PaymentMerchant(quotes.merchant.name, quotes.merchant.iconUrl),
                provider = provider,
            ).toJson(),
            direction = TransactionDirection.Outgoing,
            blockNumber = "0",
        )
    }

    private companion object {
        const val TAG = "RecordPayment"
    }
}
