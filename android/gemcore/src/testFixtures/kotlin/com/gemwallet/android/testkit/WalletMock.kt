package com.gemwallet.android.testkit

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType

fun mockWalletId(id: String = "wallet-1") = WalletId(id)

fun mockWallet(id: String = "wallet-1", name: String = "Wallet", type: WalletType = WalletType.Multicoin, accounts: List<Account> = emptyList(), source: WalletSource = WalletSource.Create) = Wallet(
    id = WalletId(id),
    name = name,
    index = 0,
    type = type,
    accounts = accounts,
    isPinned = false,
    source = source,
)

fun mockWalletMulticoin(address: String = "0xabc", name: String = "Main Wallet") = mockWallet(
    id = "multicoin_$address",
    name = name,
    accounts = listOf(mockAccount(chain = Chain.Ethereum, address = address)),
)
