package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.SessionDao
import com.gemwallet.android.data.service.store.database.entities.DbSession
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletSessionStore
import java.util.Locale

class GemstoneWalletSessionStore(
    private val sessionDao: SessionDao,
    private val preferencesService: GemPreferencesService,
) : GemWalletSessionStore {

    override fun getCurrentWalletId(): String? = sessionDao.getSession()?.walletId

    override fun setCurrentWalletId(walletId: String?) = runBlocking(Dispatchers.IO) {
        val session = sessionDao.getSession()?.copy(walletId = walletId)
            ?: DbSession(walletId = walletId, currency = defaultCurrency())
        sessionDao.update(session)
    }

    private fun defaultCurrency(): Currency =
        preferencesService.defaultCurrency(runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }.getOrNull()).decodeJson()
}
