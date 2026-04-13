package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.ObserveFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class ObserveFiatTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val fiatTransactionsDao: FiatTransactionsDao,
) : ObserveFiatTransactions {

    override fun invoke(): Flow<List<FiatTransactionAssetData>> {
        return sessionRepository.session()
            .map { it?.wallet?.id }
            .flatMapLatest { id ->
                if (id != null) {
                    fiatTransactionsDao.getFiatTransactions(id).map { it.toDTO() }
                } else {
                    flowOf(emptyList())
                }
            }
    }
}
