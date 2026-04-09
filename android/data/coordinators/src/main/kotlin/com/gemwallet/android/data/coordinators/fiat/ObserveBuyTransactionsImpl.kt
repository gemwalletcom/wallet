package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.ObserveBuyTransactions
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class ObserveBuyTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val buyRepository: BuyRepository,
) : ObserveBuyTransactions {

    override fun invoke(): Flow<List<FiatTransactionAssetData>> {
        return sessionRepository.session()
            .map { it?.wallet?.id }
            .flatMapLatest { id ->
                if (id != null) buyRepository.observeFiatTransactions(id) else flowOf(emptyList())
            }
    }
}
