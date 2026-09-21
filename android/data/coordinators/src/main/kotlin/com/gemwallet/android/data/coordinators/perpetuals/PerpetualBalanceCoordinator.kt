package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemPerpetualCollateral

private val EmptyBalance = PerpetualBalance(available = 0.0, reserved = 0.0, withdrawable = 0.0)

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualBalanceCoordinator(private val perpetualStore: GemstonePerpetualStore, private val getSession: GetSession, private val pricesDao: PricesDao) : GetPerpetualBalance {

    override fun getBalance(): Flow<PerpetualBalance?> = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualStore.observeBalance(it.wallet.id, HypercoreUSDC.id) }

    override fun getCollateral(): Flow<GemPerpetualCollateral?> = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { session ->
            combine(perpetualStore.observeBalance(session.wallet.id, HypercoreUSDC.id), pricesDao.getPrice(HypercoreUSDC.id.toIdentifier())) { balance, price ->
                balance?.let { GemPerpetualCollateral(balance = it.toGem(), price = price ?: 0.0) }
            }
        }
}
