package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.SessionDao
import com.gemwallet.android.data.service.store.database.entities.DbSession
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletSessionStore

class GemstoneWalletSessionStore(
    private val sessionDao: SessionDao,
    private val preferencesService: GemPreferencesService,
) : GemWalletSessionStore {

    override fun getCurrentWalletId(): String? = sessionDao.getSession()?.walletId

    override fun setCurrentWalletId(walletId: String?) {
        val walletId = walletId ?: return sessionDao.clearNow()
        val session = sessionDao.getSession()?.copy(walletId = walletId)
            ?: DbSession(walletId = walletId, currency = preferencesService.getCurrency().decodeJson<Currency>())
        sessionDao.updateNow(session)
    }
}
