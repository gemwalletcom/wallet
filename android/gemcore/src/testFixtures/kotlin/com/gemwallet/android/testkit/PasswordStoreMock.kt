package com.gemwallet.android.testkit

import com.gemwallet.android.application.PasswordNotFoundException
import com.gemwallet.android.application.PasswordStore

class PasswordStoreMock(
    private val passwords: MutableMap<String, String> = mutableMapOf(),
    private val readFailures: Map<String, RuntimeException> = emptyMap(),
    private val generatedPassword: String = "generated-password",
) : PasswordStore {
    var createdPasswords = 0
        private set

    override fun getOrCreatePassword(key: String): String = passwords.getOrPut(key) {
        createdPasswords += 1
        generatedPassword
    }

    override fun removePassword(key: String): Boolean = passwords.remove(key) != null

    override fun hasPassword(key: String): Boolean {
        readFailures[key]?.let { throw it }
        return passwords.containsKey(key)
    }

    override fun getPassword(key: String): String {
        readFailures[key]?.let { throw it }
        return passwords[key] ?: throw PasswordNotFoundException()
    }

    override fun putPassword(key: String, password: String) {
        passwords[key] = password
    }
}
