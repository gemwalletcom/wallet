package com.gemwallet.android.data.password

import android.content.Context
import uniffi.gemstone.GemPreferences

internal const val GEM_PREFERENCES_FILE_NAME = "gem_secure_preferences"
private const val GEM_PREFERENCES_NAMESPACE = "gateway_secure_preferences"
private const val GEM_PREFERENCES_KEYSET_NAME = "gem_secure_preferences_keyset"
internal const val GEM_PREFERENCES_KEYSET_FILE_NAME = "gem_secure_preferences_keyset_prefs"
private const val GEM_PREFERENCES_MASTER_KEY_ALIAS = "gem_secure_preferences_master_key"

private val GEM_PREFERENCES_STORE_CONFIG = TinkStoreConfig(
    preferencesFileName = GEM_PREFERENCES_FILE_NAME,
    namespace = GEM_PREFERENCES_NAMESPACE,
    keysetName = GEM_PREFERENCES_KEYSET_NAME,
    keysetPreferencesFileName = GEM_PREFERENCES_KEYSET_FILE_NAME,
    masterKeyAlias = GEM_PREFERENCES_MASTER_KEY_ALIAS,
)

class TinkGemPreferences private constructor(
    private val encryptedStore: SecureStringStore,
    private val legacyStore: SecureStringStore,
) : GemPreferences {

    constructor(context: Context) : this(
        encryptedStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = GEM_PREFERENCES_STORE_CONFIG,
        ),
        legacyStore = LegacyEncryptedPreferences(
            context = context,
            preferencesFileName = LEGACY_PREFERENCES_FILE_NAME,
        ),
    )

    override fun get(key: String): String? = encryptedStore.getOrMigrate(legacyStore, key)

    override fun set(key: String, value: String) {
        encryptedStore.putString(key, value)
        legacyStore.removeString(key)
    }

    override fun remove(key: String) {
        encryptedStore.removeString(key)
        legacyStore.removeString(key)
    }
}
