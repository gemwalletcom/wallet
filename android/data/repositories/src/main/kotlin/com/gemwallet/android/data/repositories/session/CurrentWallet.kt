package com.gemwallet.android.data.repositories.session

import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

fun SessionRepository.currentWallet(): Flow<Wallet?> = session()
    .map { it?.wallet }
    .distinctUntilChanged()
