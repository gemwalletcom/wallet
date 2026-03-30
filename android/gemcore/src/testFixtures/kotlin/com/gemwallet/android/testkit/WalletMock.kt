package com.gemwallet.android.testkit

import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType

fun mockWallet(
    id: String = "wallet-1",
    name: String = "Primary Wallet",
    index: Int = 0,
    type: WalletType = WalletType.Single,
    accounts: List<com.wallet.core.primitives.Account> = listOf(
        mockAccount(
            chain = com.wallet.core.primitives.Chain.Ethereum,
            address = "0xabc",
            derivationPath = "m/44'/60'/0'/0/0",
        )
    ),
    order: Int = 0,
    isPinned: Boolean = false,
    source: WalletSource = WalletSource.Create,
) = Wallet(
    id = id,
    name = name,
    index = index,
    type = type,
    accounts = accounts,
    order = order,
    isPinned = isPinned,
    source = source,
)
