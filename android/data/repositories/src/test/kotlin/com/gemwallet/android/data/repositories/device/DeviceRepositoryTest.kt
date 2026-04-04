package com.gemwallet.android.data.repositories.device

import com.gemwallet.android.testkit.mockDevice
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletSubscriptionChains
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DeviceRepositoryTest {

    @Test
    fun deviceHasChanges_trueWhenCurrencyChanged() {
        val remote = mockDevice(
            id = "device-id",
            token = "push-token",
            locale = "en-US",
        )
        val local = remote.copy(currency = "EUR")

        assertTrue(deviceHasChanges(remote, local))
    }

    @Test
    fun deviceHasChanges_falseWhenDevicesMatch() {
        val remote = mockDevice(
            id = "device-id",
            token = "push-token",
            locale = "en-US",
        )

        assertFalse(deviceHasChanges(remote, remote.copy()))
    }

    @Test
    fun subscriptionSyncState_invalidate_isIdempotent() {
        val initial = SubscriptionSyncState(version = 4, hasPendingChanges = false)

        val invalidated = initial.invalidate()
        val invalidatedAgain = invalidated.invalidate()

        assertEquals(5, invalidated.version)
        assertTrue(invalidated.hasPendingChanges)
        assertEquals(invalidated, invalidatedAgain)
    }

    @Test
    fun subscriptionSyncState_shouldSync_falseWhenVersionsMatchAndStateClean() {
        val state = SubscriptionSyncState(version = 4, hasPendingChanges = false)

        assertFalse(state.shouldSync(remoteVersion = 4))
    }

    @Test
    fun subscriptionSyncState_shouldSync_trueWhenVersionChanged() {
        val state = SubscriptionSyncState(version = 4, hasPendingChanges = false)

        assertTrue(state.shouldSync(remoteVersion = 5))
    }

    @Test
    fun subscriptionSyncState_shouldSync_trueWhenStateIsDirty() {
        val state = SubscriptionSyncState(version = 4, hasPendingChanges = true)

        assertTrue(state.shouldSync(remoteVersion = 4))
    }

    @Test
    fun subscriptionVersionForDeviceUpdate_preservesRemoteVersionForUnrelatedUpdate() {
        val version = subscriptionVersionForDeviceUpdate(
            localVersion = 5,
            remoteVersion = 9,
            useLocalVersion = false,
        )

        assertEquals(9, version)
    }

    @Test
    fun subscriptionVersionForDeviceUpdate_usesLocalVersionAfterSubscriptionSync() {
        val version = subscriptionVersionForDeviceUpdate(
            localVersion = 5,
            remoteVersion = 9,
            useLocalVersion = true,
        )

        assertEquals(5, version)
    }

    @Test
    fun subscriptionsDiff_emptyDiff_returnsEmpty() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Bitcoin, "bc1xyz")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertTrue(toAdd.isEmpty())
        assertTrue(toRemove.isEmpty())
    }

    @Test
    fun subscriptionsDiff_oneAddDiff_returnsOneToAdd() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Bitcoin, "bc1xyz")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertEquals(1, toAdd.size)
        assertEquals("wallet1", toAdd[0].walletId)
        assertEquals(1, toAdd[0].subscriptions.size)
        assertEquals("bc1xyz", toAdd[0].subscriptions[0].address)
        assertEquals(listOf(Chain.Bitcoin), toAdd[0].subscriptions[0].chains)
        assertTrue(toRemove.isEmpty())
    }

    @Test
    fun subscriptionsDiff_fewAddDiff_returnsFewToAdd() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Bitcoin, "bc1xyz"),
                createAccount(Chain.Solana, "solana123"),
                createAccount(Chain.Polygon, "0xabc")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertEquals(1, toAdd.size)
        assertEquals("wallet1", toAdd[0].walletId)
        assertEquals(3, toAdd[0].subscriptions.size)

        val addressChains = toAdd[0].subscriptions
        val ethPolygonChains = addressChains.find { it.address == "0xabc" }
        val bitcoinChains = addressChains.find { it.address == "bc1xyz" }
        val solanaChains = addressChains.find { it.address == "solana123" }

        assertEquals(listOf(Chain.Polygon), ethPolygonChains?.chains)
        assertEquals(listOf(Chain.Bitcoin), bitcoinChains?.chains)
        assertEquals(listOf(Chain.Solana), solanaChains?.chains)

        assertTrue(toRemove.isEmpty())
    }

    @Test
    fun subscriptionsDiff_oneRemoveDiff_returnsOneToRemove() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertTrue(toAdd.isEmpty())
        assertEquals(1, toRemove.size)
        assertEquals("wallet1", toRemove[0].walletId)
        assertEquals(listOf(Chain.Bitcoin), toRemove[0].chains)
    }

    @Test
    fun subscriptionsDiff_fewRemoveDiff_returnsFewToRemove() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin, Chain.Solana, Chain.Polygon))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertTrue(toAdd.isEmpty())
        assertEquals(1, toRemove.size)
        assertEquals("wallet1", toRemove[0].walletId)
        assertEquals(3, toRemove[0].chains.size)
        assertTrue(toRemove[0].chains.contains(Chain.Bitcoin))
        assertTrue(toRemove[0].chains.contains(Chain.Solana))
        assertTrue(toRemove[0].chains.contains(Chain.Polygon))
    }

    @Test
    fun subscriptionsDiff_bothDiffsNotEmpty_returnsBothToAddAndToRemove() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Solana, "solana123"),
                createAccount(Chain.Polygon, "0xabc")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertEquals(1, toAdd.size)
        assertEquals("wallet1", toAdd[0].walletId)
        assertEquals(2, toAdd[0].subscriptions.size)

        val addressChains = toAdd[0].subscriptions
        val ethPolygonChains = addressChains.find { it.address == "0xabc" }
        val solanaChains = addressChains.find { it.address == "solana123" }

        assertEquals(listOf(Chain.Polygon), ethPolygonChains?.chains)
        assertEquals(listOf(Chain.Solana), solanaChains?.chains)

        assertEquals(1, toRemove.size)
        assertEquals("wallet1", toRemove[0].walletId)
        assertEquals(listOf(Chain.Bitcoin), toRemove[0].chains)
    }

    @Test
    fun subscriptionsDiff_emptyWallets_returnsRemoteAsToRemove() {
        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin)),
            WalletSubscriptionChains("wallet2", listOf(Chain.Solana))
        )

        val (toAdd, toRemove) = emptyList<Wallet>().subscriptionsDiff(remote)

        assertTrue(toAdd.isEmpty())
        assertEquals(2, toRemove.size)
        assertEquals("wallet1", toRemove[0].walletId)
        assertEquals(listOf(Chain.Ethereum, Chain.Bitcoin), toRemove[0].chains)
        assertEquals("wallet2", toRemove[1].walletId)
        assertEquals(listOf(Chain.Solana), toRemove[1].chains)
    }

    @Test
    fun subscriptionsDiff_emptyRemote_returnsWalletsAsToAdd() {
        val wallets = listOf(
            createWallet(
                id = "wallet1",
                accounts = listOf(
                    createAccount(Chain.Ethereum, "0xabc"),
                    createAccount(Chain.Bitcoin, "bc1xyz")
                )
            ),
            createWallet(
                id = "wallet2",
                accounts = listOf(
                    createAccount(Chain.Solana, "solana123")
                )
            )
        )

        val (toAdd, toRemove) = wallets.subscriptionsDiff(emptyList())

        assertEquals(2, toAdd.size)

        val wallet1Add = toAdd.find { it.walletId == "wallet1" }
        assertEquals(2, wallet1Add?.subscriptions?.size)

        val wallet2Add = toAdd.find { it.walletId == "wallet2" }
        assertEquals(1, wallet2Add?.subscriptions?.size)
        assertEquals("solana123", wallet2Add?.subscriptions?.get(0)?.address)
        assertEquals(listOf(Chain.Solana), wallet2Add?.subscriptions?.get(0)?.chains)

        assertTrue(toRemove.isEmpty())
    }

    @Test
    fun walletsForSubscriptionSync_addWallet_keepsStoredWalletsInDiff() {
        val storedWallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Bitcoin, "bc1xyz")
            )
        )
        val requestedWallet = createWallet(
            id = "wallet2",
            accounts = listOf(
                createAccount(Chain.Solana, "solana123")
            )
        )
        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum, Chain.Bitcoin))
        )

        val walletsForSync = walletsForSubscriptionSync(
            storedWallets = listOf(storedWallet),
            requestedWallets = listOf(requestedWallet),
        )
        val (toAdd, toRemove) = walletsForSync.subscriptionsDiff(remote)

        assertEquals(2, walletsForSync.size)
        assertEquals(1, toAdd.size)
        assertEquals("wallet2", toAdd[0].walletId)
        assertTrue(toRemove.isEmpty())
    }

    @Test
    fun subscriptionsDiff_multipleWalletsWithSameAddress_groupsByAddress() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc"),
                createAccount(Chain.Polygon, "0xabc"),
                createAccount(Chain.Arbitrum, "0xabc")
            )
        )

        val (toAdd, _) = listOf(wallet).subscriptionsDiff(emptyList())

        assertEquals(1, toAdd.size)
        assertEquals("wallet1", toAdd[0].walletId)
        assertEquals(1, toAdd[0].subscriptions.size)

        val addressChain = toAdd[0].subscriptions[0]
        assertEquals("0xabc", addressChain.address)
        assertEquals(3, addressChain.chains.size)
        assertTrue(addressChain.chains.contains(Chain.Ethereum))
        assertTrue(addressChain.chains.contains(Chain.Polygon))
        assertTrue(addressChain.chains.contains(Chain.Arbitrum))
    }

    @Test
    fun subscriptionsDiff_walletNotInRemote_returnsToRemove() {
        val wallet = createWallet(
            id = "wallet1",
            accounts = listOf(
                createAccount(Chain.Ethereum, "0xabc")
            )
        )

        val remote = listOf(
            WalletSubscriptionChains("wallet1", listOf(Chain.Ethereum)),
            WalletSubscriptionChains("wallet2", listOf(Chain.Bitcoin))
        )

        val (toAdd, toRemove) = listOf(wallet).subscriptionsDiff(remote)

        assertTrue(toAdd.isEmpty())
        assertEquals(1, toRemove.size)
        assertEquals("wallet2", toRemove[0].walletId)
        assertEquals(listOf(Chain.Bitcoin), toRemove[0].chains)
    }

    private fun createWallet(
        id: String,
        accounts: List<Account>,
        name: String = "Test Wallet",
        type: WalletType = WalletType.Multicoin,
        source: WalletSource = WalletSource.Create
    ): Wallet {
        return Wallet(
            id = id,
            name = name,
            index = 0,
            type = type,
            accounts = accounts,
            order = 0,
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
