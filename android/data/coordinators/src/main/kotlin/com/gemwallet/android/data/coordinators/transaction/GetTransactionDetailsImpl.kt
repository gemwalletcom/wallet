package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.transactions.cases.TransactionDetails
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapNotNull
import uniffi.gemstone.GemTransactionDetailsServiceInterface

class GetTransactionDetailsImpl(private val getSession: GetSession, private val getTransaction: GetTransaction, private val transactionDetailsService: GemTransactionDetailsServiceInterface) : GetTransactionDetails {

    override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetails?> = combine(
        getSession().filterNotNull(),
        getTransaction(id),
    ) { session, data -> Pair(session, data) }
        .mapNotNull { (session, data) ->
            data?.let {
                TransactionDetails(
                    rows = transactionDetailsService.detailRows(it.toGem(), session.wallet.type.toGem()),
                    currency = session.currency,
                )
            }
        }
        .flowOn(Dispatchers.IO)
}
