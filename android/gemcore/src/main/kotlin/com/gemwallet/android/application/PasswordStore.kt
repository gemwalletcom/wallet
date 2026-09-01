package com.gemwallet.android.application

interface PasswordStore {
    fun getOrCreatePassword(key: String): String
    fun removePassword(key: String): Boolean
    fun hasPassword(key: String): Boolean
    fun getPassword(key: String): String
    fun putPassword(key: String, password: String)

    enum class Keys(val key: String) {
        Password("password"),
        DevicePrivateKey("gem_api_pk"),
        DevicePublicKey("gem_api_pb")
    }
}

class PasswordNotFoundException : IllegalStateException("Password not found")
