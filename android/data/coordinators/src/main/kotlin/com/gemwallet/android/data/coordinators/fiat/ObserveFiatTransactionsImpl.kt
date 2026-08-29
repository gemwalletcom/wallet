package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.cases.ObserveFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneFiatStore
import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class ObserveFiatTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val fiatStore: GemstoneFiatStore,
) : ObserveFiatTransactions {

    override fun invoke(): Flow<List<FiatTransactionAssetData>> {
        return sessionRepository.session()
            .map { it?.wallet?.id?.id }
            .flatMapLatest { id ->
                if (id != null) {
                    fiatStore.observeTransactions(id)
                } else {
                    flowOf(emptyList())
                }
            }
    }
}
