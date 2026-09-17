package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.PaymentAmount
import uniffi.gemstone.PaymentRequest
import java.math.BigInteger

fun mockPaymentRequest(
    address: String = mockAccount().address,
    assetId: AssetId = mockAssetId(),
    memo: String? = null,
) = PaymentRequest(
    address = address,
    amount = PaymentAmount.AtomicValue(BigInteger("19000000")),
    memo = memo,
    label = null,
    references = null,
    assetId = assetId.toIdentifier(),
)
