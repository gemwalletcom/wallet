package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetImportInProgress
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.data.repositories.session.SessionRepository
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetImportInProgressImpl(
    private val sessionRepository: SessionRepository,
    private val getImportWalletState: GetImportWalletState,
) : GetImportInProgress {

    override fun invoke(): Flow<Boolean> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { session ->
                getImportWalletState
                    .getImportState(session.wallet.id)
                    .mapLatest { it == ImportWalletState.Importing }
            }
    }
}
