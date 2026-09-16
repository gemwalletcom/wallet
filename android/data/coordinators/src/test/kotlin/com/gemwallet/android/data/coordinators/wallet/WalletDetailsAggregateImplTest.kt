package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.walletDetails

class WalletDetailsAggregateImplTest {

    private val account = Account(
        chain = Chain.Ethereum,
        address = "0x403BC00000000000000000000000000000051bDa",
        derivationPath = "m/44'/60'/0'/0/0",
    )

    private fun aggregate(
        id: String,
        type: WalletType,
        accounts: List<Account>,
    ) = WalletDetailsAggregateImpl(walletDetails(mockWallet(id = id, type = type, accounts = accounts).toGem()))

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
