package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.cases.ClearSession
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletSessionStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
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
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
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
    private val walletStore: com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore,
    private val walletSessionService: GemWalletSessionService,
    private val preferencesService: GemPreferencesService,
    private val priceService: GemPriceService,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetSession, GetCurrentWallet, GetCurrentCurrency, SetCurrentCurrency, SetCurrentWallet, ClearSession {

    private val currencyState = MutableStateFlow(preferencesService.getCurrency().decodeJson<Currency>())

    private val session: StateFlow<Session?> = sessionStore.observeSession()
        .flatMapLatest { record ->
            val walletId = record?.walletId ?: return@flatMapLatest flow { emit(null) }
            walletStore.observeWallet(WalletId(walletId)).mapLatest { wallet ->
                record.toDTO(wallet ?: return@mapLatest null)
            }
        }
        .stateIn(scope, SharingStarted.Eagerly, null)

    init {
        scope.launch {
            setCurrency(preferencesService.setupCurrency(sessionStore.storedCurrency()?.string ?: localeCurrencyCode()).decodeJson())
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
            priceService.changeCurrency(currency.toJson())
            deviceService.synchronizeIfNeeded()
        }
    }

    override suspend fun setCurrentWallet(walletId: WalletId) = withContext(Dispatchers.IO) {
        walletSessionService.setCurrentWalletId(walletId.id)
    }

    override suspend fun clearSession() = withContext(Dispatchers.IO) { sessionStore.clear() }

    private suspend fun setCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        preferencesService.setCurrency(currency.toJson())
        sessionStore.setCurrency(currency)
        currencyState.value = currency
    }

    private fun localeCurrencyCode(): String? =
        runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }.getOrNull()
}
