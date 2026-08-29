package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetImportInProgress
import com.gemwallet.android.application.wallet_import.cases.GetImportWalletState
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.application.session.cases.GetSession
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetImportInProgressImpl(
    private val getSession: GetSession,
    private val getImportWalletState: GetImportWalletState,
) : GetImportInProgress {

    override fun invoke(): Flow<Boolean> {
        return getSession()
            .filterNotNull()
            .flatMapLatest { session ->
                getImportWalletState
                    .getImportState(session.wallet.id)
                    .mapLatest { it == ImportWalletState.Importing }
            }
    }
}
