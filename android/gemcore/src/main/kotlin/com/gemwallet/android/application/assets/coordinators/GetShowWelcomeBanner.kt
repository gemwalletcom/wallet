package com.gemwallet.android.application.assets.coordinators

import kotlinx.coroutines.flow.Flow

interface GetShowWelcomeBanner {
    operator fun invoke(isWalletEmpty: Flow<Boolean>): Flow<Boolean>
}
