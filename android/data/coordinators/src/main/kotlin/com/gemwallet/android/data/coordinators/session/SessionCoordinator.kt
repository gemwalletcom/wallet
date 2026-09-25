package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletSessionStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.Session
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemCurrencyService
import uniffi.gemstone.GemCurrencyServiceInterface
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPreferencesServiceInterface
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemWalletSessionServiceInterface
import java.util.Locale

@OptIn(ExperimentalCoroutinesApi::class)
class SessionCoordinator(
    private val sessionStore: GemstoneWalletSessionStore,
    private val walletStore: GemstoneWalletStore,
    private val walletSessionService: GemWalletSessionServiceInterface,
    private val preferencesService: GemPreferencesServiceInterface,
    private val currencyService: GemCurrencyServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetSession,
    GetCurrentWallet,
    GetCurrentCurrency,
    SetCurrentCurrency {

    private val currencyState = MutableStateFlow(preferencesService.getCurrency().toPrimitives())

    private val currentWallet: Flow<Wallet?> = sessionStore.observeWalletId()
        .flatMapLatest { walletId ->
            val id = walletId ?: return@flatMapLatest flow { emit(null) }
            walletStore.observeWallet(WalletId(id))
        }

    private val session: StateFlow<Session?> = combine(currentWallet, currencyState) { wallet, currency ->
        wallet?.let { Session(wallet = it, currency = currency) }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    init {
        scope.launch {
            setCurrency(preferencesService.setupCurrency(localeCurrencyCode()).toPrimitives())
        }
    }

    override fun invoke(): StateFlow<Session?> = session

    override suspend fun getCurrentWallet(): Wallet? = withContext(Dispatchers.IO) {
        walletSessionService.getCurrentWallet()?.toPrimitives()
    }

    override fun observe(): Flow<Wallet?> = session.map { it?.wallet }.distinctUntilChanged()

    override fun getCurrency(): StateFlow<Currency> = currencyState

    override suspend fun setCurrentCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        if (currencyState.value == currency) {
            return@withContext
        }
        currencyService.setCurrency(currency.toGem())
        currencyState.value = currency
    }

    private suspend fun setCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        preferencesService.setCurrency(currency.toGem())
        currencyState.value = currency
    }

    private fun localeCurrencyCode(): String? = runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }.getOrNull()
}
