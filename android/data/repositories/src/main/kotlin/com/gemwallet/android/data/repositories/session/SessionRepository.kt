package com.gemwallet.android.data.repositories.session

import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.model.Session
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.StateFlow

interface SessionRepository : GetCurrentCurrency {

    fun session(): StateFlow<Session?>

    suspend fun getCurrentWallet(): Wallet?

    suspend fun setWallet(wallet: Wallet)

    suspend fun setCurrency(currency: Currency)

    suspend fun reset()
}
