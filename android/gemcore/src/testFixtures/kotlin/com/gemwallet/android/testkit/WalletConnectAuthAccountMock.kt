package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemWalletConnectAuthAccount

fun mockGemWalletConnectAuthAccount(
    address: String = "0xabc",
) = GemWalletConnectAuthAccount(
    account = mockAccount(chain = Chain.Ethereum, address = address).toGem(),
    chainId = "eip155:1",
    issuer = "did:pkh:eip155:1:$address",
)
