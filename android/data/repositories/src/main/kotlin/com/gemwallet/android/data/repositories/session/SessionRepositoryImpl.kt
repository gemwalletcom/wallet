package com.gemwallet.android.data.repositories.session

import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletSessionStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemPreferencesService
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
class SessionRepositoryImpl(
    private val sessionStore: GemstoneWalletSessionStore,
    private val walletStore: GemstoneWalletStore,
    private val walletSessionService: GemWalletSessionService,
    private val preferencesService: GemPreferencesService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : SessionRepository {

    private val currencyState = MutableStateFlow(preferencesService.getCurrency().decodeJson<Currency>())

    val session = sessionStore.observeSession().flatMapLatest { record ->
        val walletId = record?.walletId ?: return@flatMapLatest flow { emit(null) }
        walletStore.observeWallet(WalletId(walletId)).mapLatest { wallet ->
            record.toDTO(wallet ?: return@mapLatest null)
        }
    }
    .stateIn(scope, SharingStarted.Eagerly, null)

    init {
        scope.launch(Dispatchers.IO) {
            setCurrency(preferencesService.setupCurrency(sessionStore.storedCurrency()?.string ?: localeCurrencyCode()).decodeJson())
        }
    }

    override fun session(): StateFlow<Session?> = session

    override suspend fun getCurrentWallet(): Wallet? = withContext(Dispatchers.IO) {
        walletSessionService.getCurrentWallet()?.decodeJson<Wallet>()
    }

    override suspend fun setWallet(wallet: Wallet) = withContext(Dispatchers.IO) {
        walletSessionService.setCurrentWalletId(wallet.id.id)
    }

    override suspend fun setCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        preferencesService.setCurrency(currency.toJson())
        sessionStore.setCurrency(currency)
        currencyState.value = currency
    }

    override suspend fun reset() = withContext(Dispatchers.IO) {
        sessionStore.clear()
    }

    override suspend fun getCurrentCurrency(): Currency = currencyState.value

    override fun getCurrency(): Flow<Currency> = currencyState

    private fun localeCurrencyCode(): String? =
        runCatching { java.util.Currency.getInstance(java.util.Locale.getDefault()).currencyCode }.getOrNull()
}
