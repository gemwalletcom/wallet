package com.gemwallet.android.data.repositories.device

import com.gemwallet.android.testkit.mockDevice
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletSubscriptionChains
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class DeviceRepositoryTest {

    @Test
    fun deviceHasChanges_trueWhenCurrencyChanged() {
        val remote = mockDevice(
            id = "device-id",
            token = "push-token",
        )
        val local = remote.copy(currency = Currency.EUR)

        assertTrue(deviceHasChanges(remote, local))
    }

    @Test
    fun deviceHasChanges_falseWhenDevicesMatch() {
        val remote = mockDevice(
            id = "device-id",
            token = "push-token",
        )

        assertFalse(deviceHasChanges(remote, remote.copy()))
    }

    @Test
    fun subscriptionSignature_ignoresRenameAndPin() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(createAccount(Chain.Ethereum, "0xabc")),
        )
        val renamed = wallet.copy(name = "Renamed", isPinned = true)

        assertEquals(listOf(wallet).subscriptionSignature(), listOf(renamed).subscriptionSignature())
    }

    @Test
    fun subscriptionSignature_changesWhenAccountAdded() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(createAccount(Chain.Ethereum, "0xabc")),
        )
        val extended = wallet.copy(accounts = wallet.accounts + createAccount(Chain.Bitcoin, "bc1xyz"))

        assertNotEquals(listOf(wallet).subscriptionSignature(), listOf(extended).subscriptionSignature())
    }

    @Test
    fun subscriptionSignature_isOrderIndependent() {
        val first = createWallet(id = "wallet1", accounts = listOf(createAccount(Chain.Ethereum, "0xabc")))
        val second = createWallet(id = "wallet2", accounts = listOf(createAccount(Chain.Solana, "solana123")))

        assertEquals(listOf(first, second).subscriptionSignature(), listOf(second, first).subscriptionSignature())
    }

    private fun createWallet(
        id: String,
        accounts: List<Account>,
        name: String = "Test Wallet",
        type: WalletType = WalletType.Multicoin,
        source: WalletSource = WalletSource.Create
    ): Wallet {
        return Wallet(
            id = WalletId(id),
            name = name,
            index = 0,
            type = type,
            accounts = accounts,
            isPinned = false,
            source = source
        )
    }

    private fun createAccount(
        chain: Chain,
        address: String,
        derivationPath: String = "m/44'/60'/0'/0/0"
    ): Account {
        return Account(
            chain = chain,
            address = address,
            derivationPath = derivationPath
        )
    }
}
