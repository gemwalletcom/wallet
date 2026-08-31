package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

interface HasMultiSign {
    fun hasMultiSign(wallet: Wallet): Flow<Boolean>
}