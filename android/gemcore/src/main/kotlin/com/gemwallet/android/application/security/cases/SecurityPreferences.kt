package com.gemwallet.android.application.security.cases

import kotlinx.coroutines.flow.Flow

interface SecurityPreferences {
    fun authRequired(): Boolean

    fun setAuthRequired(enabled: Boolean)

    fun getLockInterval(): Flow<Int>

    suspend fun setLockInterval(minutes: Int)
}
