package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemWalletPreferencesService
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance as PerpetualBalanceDisplay
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn

private val EmptyBalance = PerpetualBalance(available = 0.0, reserved = 0.0, withdrawable = 0.0)

@OptIn(ExperimentalCoroutinesApi::class)
class PerpetualBalanceCoordinator(
    private val perpetualStore: GemstonePerpetualStore,
    private val getSession: GetSession,
    private val observePerpetualWallet: ObservePerpetualWallet,
    private val walletPreferencesService: GemWalletPreferencesService,
) : GetPerpetualBalance {

    override fun getBalance(): Flow<PerpetualBalance?> = getSession()
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { perpetualStore.observeBalance(it.wallet.id, HypercoreUSDC.id) }

    override fun getDisplayBalance(): Flow<PerpetualBalanceDisplay> =
        getBalance().map { PerpetualBalanceDisplayValue(it ?: EmptyBalance) }

    override fun getCollateralIncludedInTotal(): Flow<PerpetualBalance?> = observePerpetualWallet()
        .flatMapLatest { wallet ->
            when {
                wallet == null -> flowOf(null)
                !walletPreferencesService.includesPerpetualCollateral(wallet.id.id) -> flowOf(null)
                else -> perpetualStore.observeBalance(wallet.id, HypercoreUSDC.id)
            }
        }
        .flowOn(Dispatchers.IO)
}

private class PerpetualBalanceDisplayValue(val balance: PerpetualBalance) : PerpetualBalanceDisplay {
    private val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
    override val deposit: String get() = formatter.string(balance.reserved)
    override val available: String get() = formatter.string(balance.available)
    override val withdrawable: String get() = formatter.string(balance.withdrawable)
    override val total: String get() = formatter.string(balance.available + balance.reserved)
}
