package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalances
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance as PerpetualBalanceUi
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.walletId
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map

private val EmptyBalance = PerpetualBalance(available = 0.0, reserved = 0.0, withdrawable = 0.0)

@OptIn(ExperimentalCoroutinesApi::class)
class GetPerpetualBalancesImpl(
    private val sessionRepository: SessionRepository,
    private val perpetualRepository: PerpetualRepository,
) : GetPerpetualBalances {

    override fun getPerpetualBalance(): Flow<PerpetualBalanceUi> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { perpetualRepository.getBalance(it.wallet.walletId, HypercoreUSDC.id) }
            .map { PerpetualBalanceImpl(it ?: EmptyBalance) }
    }
}

class PerpetualBalanceImpl(val balance: PerpetualBalance) : PerpetualBalanceUi {
    override val deposit: String get() = Currency.USD.format(balance.reserved)
    override val available: String get() = Currency.USD.format(balance.available)
    override val withdrawable: String get() = Currency.USD.format(balance.withdrawable)
    override val total: String get() = Currency.USD.format(balance.available + balance.reserved)
}
