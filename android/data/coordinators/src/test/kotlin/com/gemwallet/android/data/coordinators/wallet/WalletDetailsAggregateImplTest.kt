package com.gemwallet.android.data.coordinators.wallet

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test

class WalletDetailsAggregateImplTest {

    @Test
    fun accounts_preservesChainAndAddress() {
        val wallet = Wallet(
            id = WalletId("wallet-id"),
            name = "Wallet",
            index = 0,
            type = WalletType.Single,
            accounts = listOf(
                Account(
                    chain = Chain.Ethereum,
                    address = "0x403BC00000000000000000000000000000051bDa",
                    derivationPath = "m/44'/60'/0'/0/0",
                ),
            ),
            isPinned = false,
            source = WalletSource.Create,
        )

        val aggregate = WalletDetailsAggregateImpl(wallet)

        assertEquals(Chain.Ethereum, aggregate.accounts.single().chain)
        assertEquals("0x403BC00000000000000000000000000000051bDa", aggregate.accounts.single().address)
    }
}
