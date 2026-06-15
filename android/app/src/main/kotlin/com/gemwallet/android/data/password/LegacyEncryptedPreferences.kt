@file:Suppress("DEPRECATION")

package com.gemwallet.android.data.password

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import java.io.File

// Passwords and Gemstone secure preferences historically shared this file; their keys must stay disjoint.
internal const val LEGACY_PREFERENCES_FILE_NAME = "pwd"

internal class LegacyEncryptedPreferences(
    context: Context,
    private val preferencesFileName: String,
) : SecureStringStore {

    private val context = context.applicationContext

    @Volatile
    private var sharedPreferences: SharedPreferences? = null

    override fun contains(key: String): Boolean = existingPreferences()?.contains(key) == true

    override fun getString(key: String): String? = existingPreferences()?.getString(key, null)

    override fun putString(key: String, value: String) {
        if (!preferences().edit().putString(key, value).commit()) {
            throw IllegalStateException("Legacy secure value write failed")
        }
    }

    override fun removeString(key: String): Boolean = existingPreferences()?.edit()?.remove(key)?.commit() != false

    private fun preferences(): SharedPreferences {
        sharedPreferences?.let { return it }
        return synchronized(this) {
            sharedPreferences ?: createPreferences().also { sharedPreferences = it }
        }
    }

    private fun existingPreferences(): SharedPreferences? {
        if (sharedPreferences == null && !preferencesFile.exists()) {
            return null
        }
        return preferences()
    }

    private fun createPreferences(): SharedPreferences {
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        return EncryptedSharedPreferences.create(
            context,
            preferencesFileName,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    private val preferencesFile: File
        get() = File(context.applicationInfo.dataDir, "shared_prefs/$preferencesFileName.xml")
}
