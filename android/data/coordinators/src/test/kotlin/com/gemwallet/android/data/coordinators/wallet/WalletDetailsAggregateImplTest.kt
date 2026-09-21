package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemWalletSecretKind
import com.gemwallet.android.testkit.mockGemWalletRow
import uniffi.gemstone.ChainAddress
import uniffi.gemstone.GemWalletDetails

class WalletDetailsAggregateImplTest {

    private val account = mockAccount(chain = Chain.Ethereum, address = "0x403BC00000000000000000000000000000051bDa")

    private fun aggregate(id: String, type: WalletType, accounts: List<Account>) = WalletDetailsAggregateImpl(
        GemWalletDetails(
            row = mockGemWalletRow(id = id),
            secretKind = if (type == WalletType.View) null else GemWalletSecretKind.PHRASE,
            address = accounts.singleOrNull()?.let { ChainAddress(it.chain.string, it.address) },
            addressExplorer = null,
        ),
    )

    @Test
    fun singleAccountWallet_showsItsAddressAndSecret() {
        val aggregate = aggregate("single_ethereum_${account.address}", WalletType.Single, listOf(account))

        assertEquals(WalletId("single_ethereum_${account.address}"), aggregate.id)
        assertEquals(GemWalletSecretKind.PHRASE, aggregate.secretKind)
        assertEquals(Chain.Ethereum, aggregate.address?.chain)
        assertEquals(account.address, aggregate.address?.address)
    }

    @Test
    fun multicoinWallet_hasNoSingleAddress() {
        val aggregate = aggregate(
            "multicoin_1",
            WalletType.Multicoin,
            listOf(account, account.copy(chain = Chain.Bitcoin, address = "bc1qaddress")),
        )

        assertNull(aggregate.address)
    }
}
