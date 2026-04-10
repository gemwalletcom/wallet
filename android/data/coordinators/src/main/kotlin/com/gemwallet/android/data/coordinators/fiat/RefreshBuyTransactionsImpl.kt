package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.RefreshBuyTransactions
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import kotlinx.coroutines.flow.first

class RefreshBuyTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val buyRepository: BuyRepository,
) : RefreshBuyTransactions {

    override suspend fun invoke() {
        val wallet = sessionRepository.session().first()?.wallet ?: return
        buyRepository.updateFiatTransactions(wallet)
    }
}
