package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetPerpetualBalanceImpl(
    private val perpetualStore: GemstonePerpetualStore,
    private val sessionRepository: SessionRepository,
) : GetPerpetualBalance {
    override fun getBalance(): Flow<PerpetualBalance?> = sessionRepository.session()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualStore.observeBalance(it.wallet.id, HypercoreUSDC.id) }
}
