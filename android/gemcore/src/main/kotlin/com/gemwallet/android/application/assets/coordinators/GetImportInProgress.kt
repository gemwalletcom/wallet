package com.gemwallet.android.application.assets.coordinators

import kotlinx.coroutines.flow.Flow

interface GetImportInProgress {
    operator fun invoke(): Flow<Boolean>
}
