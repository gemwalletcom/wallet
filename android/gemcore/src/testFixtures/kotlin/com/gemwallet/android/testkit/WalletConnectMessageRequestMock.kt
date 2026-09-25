package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSession
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage

fun mockGemWalletConnectMessageRequest(
    wallet: Wallet = mockWallet(id = mockWalletId(address = "0xabc"), name = "Main Wallet", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"))),
    session: WalletConnectionSession = mockWalletConnectionSession(),
    message: SignMessage = mockSignMessage(chain = Chain.Ethereum.string, signType = SignDigestType.EIP191, data = "Sign in".toByteArray()),
) = GemWalletConnectMessageRequest(
    sessionId = session.sessionId,
    chain = Chain.Ethereum.string,
    wallet = wallet.toGem(),
    account = wallet.accounts.first().toGem(),
    session = session.toGem(),
    simulation = mockSimulationResult(),
    message = message,
    assets = emptyList(),
)
