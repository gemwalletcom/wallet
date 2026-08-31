package com.gemwallet.android.application.assets.cases

import kotlinx.coroutines.flow.Flow

interface GetShowWelcomeBanner {
    operator fun invoke(): Flow<Boolean>
}
