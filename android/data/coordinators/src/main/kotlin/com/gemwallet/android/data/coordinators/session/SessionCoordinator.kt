package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletSessionStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
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
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemWalletSessionService
import java.util.Locale

@OptIn(ExperimentalCoroutinesApi::class)
class SessionCoordinator(
    private val sessionStore: GemstoneWalletSessionStore,
    private val walletStore: GemstoneWalletStore,
    private val walletSessionService: GemWalletSessionService,
    private val preferencesService: GemPreferencesService,
    private val priceService: GemPriceService,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetSession, GetCurrentWallet, GetCurrentCurrency, SetCurrentCurrency, SetCurrentWallet {

    private val currencyState = MutableStateFlow(preferencesService.getCurrency().toCurrency())

    private val currentWallet: Flow<Wallet?> = sessionStore.observeSession()
        .flatMapLatest { record ->
            val walletId = record?.walletId ?: return@flatMapLatest flow { emit(null) }
            walletStore.observeWallet(WalletId(walletId))
        }

    private val session: StateFlow<Session?> = combine(currentWallet, currencyState) { wallet, currency ->
        wallet?.let { Session(wallet = it, currency = currency) }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    init {
        scope.launch {
            setCurrency(preferencesService.setupCurrency(sessionStore.storedCurrency()?.toGem() ?: localeCurrencyCode()).toCurrency())
        }
    }

    override fun invoke(): StateFlow<Session?> = session

    override suspend fun getCurrentWallet(): Wallet? = withContext(Dispatchers.IO) {
        walletSessionService.getCurrentWallet()?.decodeJson<Wallet>()
    }

    override fun observe(): Flow<Wallet?> = session.map { it?.wallet }.distinctUntilChanged()

    override suspend fun getCurrentCurrency(): Currency = currencyState.value

    override fun getCurrency(): Flow<Currency> = currencyState

    override fun setCurrentCurrency(currency: Currency) {
        scope.launch {
            if (currencyState.value == currency) {
                return@launch
            }
            setCurrency(currency)
            priceService.changeCurrency(currency.toGem())
            deviceService.synchronizeIfNeeded()
        }
    }

    override suspend fun setCurrentWallet(walletId: WalletId) = withContext(Dispatchers.IO) {
        walletSessionService.setCurrentWalletId(walletId.id)
    }

    private suspend fun setCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        preferencesService.setCurrency(currency.toGem())
        currencyState.value = currency
    }

    private fun localeCurrencyCode(): String? =
        runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }.getOrNull()
}
