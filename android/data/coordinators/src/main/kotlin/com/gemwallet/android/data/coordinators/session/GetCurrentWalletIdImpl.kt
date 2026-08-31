package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map

class GetCurrentWalletIdImpl(
    private val getSession: GetSession,
) : GetCurrentWalletId {

    override fun invoke(): Flow<WalletId> = getSession()
        .filterNotNull()
        .map { it.wallet.id }
        .distinctUntilChanged()
}
