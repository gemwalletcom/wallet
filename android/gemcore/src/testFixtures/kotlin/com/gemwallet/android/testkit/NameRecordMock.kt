package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord

fun mockNameRecord(
    name: String = "example.eth",
    chain: Chain = Chain.Ethereum,
    address: String = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a",
    provider: NameProvider = NameProvider.Ens,
) = NameRecord(
    name = name,
    chain = chain,
    address = address,
    provider = provider,
)
