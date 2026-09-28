package com.gemwallet.android.data.services.store.database.di

import com.gemwallet.android.testkit.mockAccount
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class Migration_63_64WalletIdTest {

    @Test
    fun `migratedWalletId for Multicoin wallet type`() {
        val result = migratedWalletId(WalletType.Multicoin, Chain.Ethereum, "0x1234567890abcdef")
        assertEquals("multicoin_0x1234567890abcdef", result.id)
    }

    @Test
    fun `migratedWalletId for Multicoin ignores chain parameter`() {
        val address = "0xabcdef1234567890"
        val resultEthereum = migratedWalletId(WalletType.Multicoin, Chain.Ethereum, address)
        val resultBitcoin = migratedWalletId(WalletType.Multicoin, Chain.Bitcoin, address)
        assertEquals(resultEthereum, resultBitcoin)
        assertEquals("multicoin_$address", resultEthereum.id)
    }

    @Test
    fun `migratedWalletId for Single wallet type with Ethereum`() {
        val result = migratedWalletId(WalletType.Single, Chain.Ethereum, "0x1234567890abcdef")
        assertEquals("single_ethereum_0x1234567890abcdef", result.id)
    }

    @Test
    fun `migratedWalletId for Single wallet type with Bitcoin`() {
        val result = migratedWalletId(WalletType.Single, Chain.Bitcoin, "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")
        assertEquals("single_bitcoin_bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh", result.id)
    }

    @Test
    fun `migratedWalletId for Single wallet type with Solana`() {
        val result = migratedWalletId(WalletType.Single, Chain.Solana, "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK")
        assertEquals("single_solana_DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK", result.id)
    }

    @Test
    fun `migratedWalletId for PrivateKey wallet type with Ethereum`() {
        val result = migratedWalletId(WalletType.PrivateKey, Chain.Ethereum, "0xabcdef1234567890")
        assertEquals("privateKey_ethereum_0xabcdef1234567890", result.id)
    }

    @Test
    fun `migratedWalletId for PrivateKey wallet type with Polygon`() {
        val result = migratedWalletId(WalletType.PrivateKey, Chain.Polygon, "0x9876543210fedcba")
        assertEquals("privateKey_polygon_0x9876543210fedcba", result.id)
    }

    @Test
    fun `migratedWalletId for View wallet type with Ethereum`() {
        val result = migratedWalletId(WalletType.View, Chain.Ethereum, "0xfedcba0987654321")
        assertEquals("view_ethereum_0xfedcba0987654321", result.id)
    }

    @Test
    fun `migratedWalletId for View wallet type with Tron`() {
        val result = migratedWalletId(WalletType.View, Chain.Tron, "TRX123456789")
        assertEquals("view_tron_TRX123456789", result.id)
    }

    @Test
    fun `migratedWalletId handles different chains correctly`() {
        val address = "test_address"
        val chains = listOf(
            Chain.Ethereum to "ethereum",
            Chain.Bitcoin to "bitcoin",
            Chain.Solana to "solana",
            Chain.Polygon to "polygon",
            Chain.Arbitrum to "arbitrum",
            Chain.Optimism to "optimism",
            Chain.Base to "base",
            Chain.Tron to "tron",
        )

        chains.forEach { (chain, expectedChainString) ->
            val result = migratedWalletId(WalletType.Single, chain, address)
            assertEquals("single_${expectedChainString}_$address", result.id)
        }
    }

    @Test
    fun `migratedWalletId throws exception for empty address`() {
        val exception = assertThrows(IllegalArgumentException::class.java) {
            migratedWalletId(WalletType.Multicoin, Chain.Ethereum, "")
        }
        assertEquals("Account address cannot be empty", exception.message)
    }

    @Test
    fun `migratedWalletId for all WalletTypes maintains consistent format`() {
        val address = "0x1234567890abcdef"
        val chain = Chain.Ethereum

        val multicoin = migratedWalletId(WalletType.Multicoin, chain, address)
        val single = migratedWalletId(WalletType.Single, chain, address)
        val privateKey = migratedWalletId(WalletType.PrivateKey, chain, address)
        val view = migratedWalletId(WalletType.View, chain, address)

        assertEquals("multicoin_$address", multicoin.id)
        assertEquals("single_ethereum_$address", single.id)
        assertEquals("privateKey_ethereum_$address", privateKey.id)
        assertEquals("view_ethereum_$address", view.id)
    }

    @Test
    fun `priorityAccount returns Ethereum account when it exists`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
            mockAccount(chain = Chain.Ethereum, address = "eth123"),
            mockAccount(chain = Chain.Solana, address = "sol123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Ethereum, result?.chain)
        assertEquals("eth123", result?.address)
    }

    @Test
    fun `priorityAccount returns first Ethereum account when multiple exist`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
            mockAccount(chain = Chain.Ethereum, address = "eth_first"),
            mockAccount(chain = Chain.Ethereum, address = "eth_second"),
        )
        val result = priorityAccount(accounts)
        assertEquals("eth_first", result?.address)
    }

    @Test
    fun `priorityAccount returns first account when no Ethereum exists`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
            mockAccount(chain = Chain.Solana, address = "sol123"),
            mockAccount(chain = Chain.Polygon, address = "poly123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Bitcoin, result?.chain)
        assertEquals("btc123", result?.address)
    }

    @Test
    fun `priorityAccount returns Ethereum when it is first in list`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Ethereum, address = "eth123"),
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
            mockAccount(chain = Chain.Solana, address = "sol123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Ethereum, result?.chain)
        assertEquals("eth123", result?.address)
    }

    @Test
    fun `priorityAccount returns Ethereum when it is last in list`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
            mockAccount(chain = Chain.Solana, address = "sol123"),
            mockAccount(chain = Chain.Ethereum, address = "eth123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Ethereum, result?.chain)
        assertEquals("eth123", result?.address)
    }

    @Test
    fun `priorityAccount handles single Ethereum account`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Ethereum, address = "eth123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Ethereum, result?.chain)
        assertEquals("eth123", result?.address)
    }

    @Test
    fun `priorityAccount handles single non-Ethereum account`() {
        val accounts = listOf(
            mockAccount(chain = Chain.Bitcoin, address = "btc123"),
        )
        val result = priorityAccount(accounts)
        assertEquals(Chain.Bitcoin, result?.chain)
        assertEquals("btc123", result?.address)
    }

    @Test
    fun `priorityAccount throws exception for empty list`() {
        val exception = assertThrows(IllegalArgumentException::class.java) {
            priorityAccount(emptyList())
        }
        assertEquals("Accounts list cannot be empty", exception.message)
    }
}
