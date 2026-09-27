package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.PasswordNotFoundException
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.testkit.PasswordStoreMock
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.GemKeystoreAuthentication

private const val LEGACY_WALLET_ID = "multicoin_legacy"
private const val FIRST_NEW_WALLET_ID = "multicoin_new_1"
private const val SECOND_NEW_WALLET_ID = "multicoin_new_2"
private const val LEGACY_WALLET_PASSWORD = "0x0102"
private const val APP_PASSWORD = "0x0304"

class KeystorePasswordStoreTest {

    private fun deviceAuthentication(authRequired: Boolean = false, hasBiometrics: Boolean = false) = DeviceAuthentication(
        securityPreferences = mockk<SecurityPreferences> { every { authRequired() } returns authRequired },
        hasBiometrics = { hasBiometrics },
    )

    @Test
    fun theKeystoreReportsHowAConfirmationAuthenticates() {
        assertEquals(GemKeystoreAuthentication.NONE, GemstoneKeystorePassword(PasswordStoreMock(), deviceAuthentication(authRequired = false, hasBiometrics = true)).authentication())
        assertEquals(GemKeystoreAuthentication.BIOMETRICS, GemstoneKeystorePassword(PasswordStoreMock(), deviceAuthentication(authRequired = true, hasBiometrics = true)).authentication())
        assertEquals(GemKeystoreAuthentication.PASSCODE, GemstoneKeystorePassword(PasswordStoreMock(), deviceAuthentication(authRequired = true, hasBiometrics = false)).authentication())
    }

    @Test
    fun sharedPasswordIsCreatedOnceAndReusedWithoutWalletAliases() {
        val passwordStore = PasswordStoreMock(generatedPassword = APP_PASSWORD)
        val keystorePassword = GemstoneKeystorePassword(passwordStore, deviceAuthentication())

        val first = keystorePassword.getPassword(true)
        val second = keystorePassword.getPassword(true)

        assertEquals(APP_PASSWORD, first)
        assertEquals(first, second)
        assertEquals(1, passwordStore.createdPasswords)
        assertFalse(passwordStore.hasPassword(FIRST_NEW_WALLET_ID))
    }

    @Test
    fun readingTheSharedPasswordFailsClosedWhenItIsMissing() {
        assertThrows(PasswordNotFoundException::class.java) {
            GemstoneKeystorePassword(PasswordStoreMock(), deviceAuthentication()).getPassword(false)
        }
    }

    @Test
    fun walletPasswordIsReadForLegacyEntriesAndNullOtherwise() {
        val passwordStore = PasswordStoreMock(
            mutableMapOf(
                LEGACY_WALLET_ID to LEGACY_WALLET_PASSWORD,
                PasswordStore.Keys.Password.key to APP_PASSWORD,
            ),
        )
        val keystorePassword = GemstoneKeystorePassword(passwordStore, deviceAuthentication())

        assertEquals(LEGACY_WALLET_PASSWORD, keystorePassword.getWalletPassword(LEGACY_WALLET_ID))
        assertEquals(null, keystorePassword.getWalletPassword(FIRST_NEW_WALLET_ID))

        keystorePassword.deleteWalletPassword(LEGACY_WALLET_ID)

        assertEquals(null, keystorePassword.getWalletPassword(LEGACY_WALLET_ID))
        assertEquals(APP_PASSWORD, passwordStore.getPassword(PasswordStore.Keys.Password.key))
    }

    @Test
    fun storageFailureIsNotSwallowedAsAMissingWalletPassword() {
        val storageError = IllegalStateException("secure storage unavailable")
        val passwordStore = PasswordStoreMock(
            passwords = mutableMapOf(PasswordStore.Keys.Password.key to APP_PASSWORD),
            readFailures = mapOf(LEGACY_WALLET_ID to storageError),
        )

        val thrown = assertThrows(IllegalStateException::class.java) {
            GemstoneKeystorePassword(passwordStore, deviceAuthentication()).getWalletPassword(LEGACY_WALLET_ID)
        }

        assertSame(storageError, thrown)
    }
}
