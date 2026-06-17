package com.gemwallet.android.data.password

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.math.append0x
import com.gemwallet.android.math.hex
import java.security.SecureRandom

internal const val PASSWORD_STORE_PREFERENCES_FILE_NAME = "gem_wallet_passwords"
private const val PASSWORD_STORE_NAMESPACE = "wallet_password"
private const val PASSWORD_STORE_KEYSET_NAME = "gem_wallet_password_keyset"
internal const val PASSWORD_STORE_KEYSET_PREFERENCES_FILE_NAME = "gem_wallet_password_keyset_prefs"
private const val PASSWORD_STORE_MASTER_KEY_ALIAS = "gem_wallet_password_master_key"

private val PASSWORD_STORE_CONFIG = TinkStoreConfig(
    preferencesFileName = PASSWORD_STORE_PREFERENCES_FILE_NAME,
    namespace = PASSWORD_STORE_NAMESPACE,
    keysetName = PASSWORD_STORE_KEYSET_NAME,
    keysetPreferencesFileName = PASSWORD_STORE_KEYSET_PREFERENCES_FILE_NAME,
    masterKeyAlias = PASSWORD_STORE_MASTER_KEY_ALIAS,
)

class TinkPasswordStore internal constructor(
    private val encryptedStore: SecureStringStore,
    private val legacyStore: SecureStringStore,
    private val random: SecureRandom,
) : PasswordStore {

    constructor(context: Context) : this(
        encryptedStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = PASSWORD_STORE_CONFIG,
        ),
        legacyStore = LegacyEncryptedPreferences(
            context = context,
            preferencesFileName = LEGACY_PREFERENCES_FILE_NAME,
        ),
        random = SecureRandom(),
    )

    override fun createPassword(key: String): String {
        val password = ByteArray(32)
        random.nextBytes(password)
        val value = password.hex.append0x()
        encryptedStore.putString(key, value)
        legacyStore.removeString(key)
        return value
    }

    override fun removePassword(key: String): Boolean =
        encryptedStore.removeString(key) and legacyStore.removeString(key)

    override fun getPassword(key: String): String =
        encryptedStore.getOrMigrate(legacyStore, key) ?: throw IllegalStateException("Password not found")

    override fun putPassword(key: String, password: String) {
        encryptedStore.putString(key, password)
        legacyStore.removeString(key)
    }
}
