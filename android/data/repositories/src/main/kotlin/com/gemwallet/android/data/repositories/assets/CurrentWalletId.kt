package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.repositories.session.SessionRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map

internal fun SessionRepository.currentWalletId(): Flow<String> = session()
    .filterNotNull()
    .map { it.wallet.id.id }
    .distinctUntilChanged()
