package com.gemwallet.android.data.service.walletconnect.reown

import com.gemwallet.android.application.wallet_connect.toSupportedNamespaces
import com.gemwallet.android.data.service.walletconnect.reown.testkit.mockSessionProposal
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemSessionApproval

class TronNamespacesTest {
    private val account = mockAccount(chain = Chain.Tron, address = "address")
    private val supportedNamespaces = GemSessionApproval(
        chains = listOf("tron"),
        accounts = listOf(account.toGem()),
        methods = listOf("tron_signMessage"),
        events = emptyList(),
    ).toSupportedNamespaces(GemChainService()).mapValues { (_, namespace) ->
        Wallet.Model.Namespace.Session(
            chains = namespace.chains,
            accounts = namespace.accounts,
            methods = namespace.methods,
            events = namespace.events,
        )
    }

    @Test
    fun approvesRequestedReference() {
        for (chainId in listOf("tron:728126428", "tron:0x2b6653dc")) {
            for (optional in listOf(false, true)) {
                val namespaces = WalletKit.generateApprovedNamespaces(mockSessionProposal(chainId, optional), supportedNamespaces)
                assertEquals(listOf(chainId), namespaces["tron"]?.chains)
                assertEquals(listOf("$chainId:${account.address}"), namespaces["tron"]?.accounts)
            }
        }
    }
}
