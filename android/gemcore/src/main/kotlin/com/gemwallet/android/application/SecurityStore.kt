package com.gemwallet.android.application

interface SecurityStore<T> {
    suspend fun getValue(key: T): String
}
