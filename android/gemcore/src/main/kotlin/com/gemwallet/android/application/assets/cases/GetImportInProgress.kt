package com.gemwallet.android.application.assets.cases

import kotlinx.coroutines.flow.Flow

interface GetImportInProgress {
    operator fun invoke(): Flow<Boolean>
}
