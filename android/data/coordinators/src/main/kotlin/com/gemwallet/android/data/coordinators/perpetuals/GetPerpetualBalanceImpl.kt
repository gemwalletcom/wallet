package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
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
    private val perpetualRepository: PerpetualRepository,
    private val sessionRepository: SessionRepository,
) : GetPerpetualBalance {
    override fun getBalance(): Flow<PerpetualBalance?> = sessionRepository.session()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualRepository.getBalance(it.wallet.id, HypercoreUSDC.id) }
}
