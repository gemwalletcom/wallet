package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage

fun mockSignMessage(
    data: ByteArray = "Sign in".toByteArray(),
) = SignMessage(
    chain = Chain.Ethereum.string,
    signType = SignDigestType.EIP191,
    data = data,
)
