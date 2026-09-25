package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.transactions.cases.TransactionDetails
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemTransactionDetailsServiceInterface

class GetTransactionDetailsImpl(private val getSession: GetSession, private val transactionStore: GemstoneTransactionStore, private val transactionDetailsService: GemTransactionDetailsServiceInterface) : GetTransactionDetails {

    @OptIn(ExperimentalCoroutinesApi::class)
    override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetails?> = getSession()
        .flatMapLatest { session ->
            session ?: return@flatMapLatest flowOf(null)
            transactionStore.observeTransaction(session.wallet.id, id).map { data ->
                data?.let {
                    TransactionDetails(rows = transactionDetailsService.detailRows(it.toGem(), session.wallet.type.toGem()))
                }
            }
        }
        .flowOn(Dispatchers.IO)
}
