package com.gemwallet.android.data.services.gemstone.config

import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.data.services.store.ConfigStore
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemSecureStore
import uniffi.gemstone.lockPeriodFromMinutes

class AppLockPreferences(private val configStore: ConfigStore, private val secureStore: GemSecureStore, private val passwordProtection: WalletPasswordProtection, private val ioDispatcher: CoroutineDispatcher) : SecurityPreferences {

    private val authRequiredState by lazy { MutableStateFlow(read(SecureKey.Auth)?.toBooleanStrictOrNull() ?: configStore.getBoolean(ConfigKey.Auth.string)) }
    private val lockIntervalState by lazy { MutableStateFlow(read(SecureKey.LockInterval)?.toIntOrNull() ?: defaultLockInterval) }

    override fun authRequired(): Boolean = authRequiredState.value

    override fun setAuthRequired(enabled: Boolean) {
        secureStore.set(SecureKey.Auth.string, enabled.toString())
        authRequiredState.value = enabled
    }

    override fun isLockEnabled(): Boolean = authRequired() || passwordProtection.authenticationRequired()

    override fun getLockEnabled(): Flow<Boolean> = flow { emitAll(authRequiredState) }
        .map { it || passwordProtection.authenticationRequired() }
        .flowOn(ioDispatcher)

    override fun lockInterval(): Int = lockIntervalState.value

    override fun getLockInterval(): Flow<Int> = flow { emitAll(lockIntervalState) }
        .flowOn(ioDispatcher)

    override suspend fun setLockInterval(minutes: Int) {
        secureStore.set(SecureKey.LockInterval.string, minutes.toString())
        lockIntervalState.value = minutes
    }

    private fun read(key: SecureKey): String? = secureStore.get(key.string)

    private val defaultLockInterval: Int
        get() = lockPeriodFromMinutes(null).minutes().toInt()

    private enum class ConfigKey(val string: String) {
        Auth("auth"),
    }

    private enum class SecureKey(val string: String) {
        Auth("auth_required"),
        LockInterval("lock_interval"),
    }

    companion object {
        const val CONFIG_FILE_NAME = "config"
    }
}
