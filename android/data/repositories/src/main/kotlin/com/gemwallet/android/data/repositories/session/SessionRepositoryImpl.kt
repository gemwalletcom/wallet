package com.gemwallet.android.data.repositories.session

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.SessionDao
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

@OptIn(ExperimentalCoroutinesApi::class)
class SessionRepositoryImpl(
    private val sessionDao: SessionDao,
    private val walletsRepository: WalletsRepository,
    private val walletSessionService: GemWalletSessionService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : SessionRepository {

    val session = sessionDao.session().flatMapLatest { record ->
        val walletId = record?.walletId ?: return@flatMapLatest flow { emit(null) }
        walletsRepository.getWallet(WalletId(walletId)).mapLatest { wallet ->
            record.toDTO(wallet ?: return@mapLatest null)
        }
    }
    .stateIn(scope, SharingStarted.Eagerly, null)

    override fun session(): StateFlow<Session?> = session

    override suspend fun getCurrentWallet(): Wallet? = withContext(Dispatchers.IO) {
        walletSessionService.getCurrentWallet()?.decodeJson<Wallet>()
    }

    override suspend fun setWallet(wallet: Wallet) = withContext(Dispatchers.IO) {
        walletSessionService.setCurrentWalletId(wallet.id.id)
    }

    override suspend fun setCurrency(currency: Currency) = withContext(Dispatchers.IO) {
        sessionDao.setCurrency(currency)
    }

    override suspend fun reset() = withContext(Dispatchers.IO) {
        sessionDao.clear()
    }

    override suspend fun getCurrentCurrency(): Currency = withContext(Dispatchers.IO) {
        sessionDao.getCurrency() ?: Currency.USD
    }

    override fun getCurrency(): Flow<Currency> = session().map { it?.currency ?: Currency.USD }
}
