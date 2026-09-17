package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord

fun mockNameRecord() = NameRecord(
    name = "vitalik.eth",
    chain = Chain.Ethereum,
    address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    provider = NameProvider.Ens,
)
