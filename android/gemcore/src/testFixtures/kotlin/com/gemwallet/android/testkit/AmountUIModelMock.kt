package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.model.AssetPriceValue
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemTransactionHeaderKind
import uniffi.gemstone.PaymentPrice
import java.math.BigInteger

fun mockAmountUIModel(fromAsset: AssetPriceValue = mockAssetPriceValue(asset = mockAssetSolana()), paymentPrice: PaymentPrice? = null) = AmountUIModel(
    headerKind = GemTransactionHeaderKind.Amount(showsFiat = true),
    amount = BigInteger("1000000000"),
    fromAsset = fromAsset,
    toAsset = null,
    fromAmount = BigInteger("1000000000"),
    toAmount = null,
    nftAsset = null,
    currency = Currency.USD,
    paymentPrice = paymentPrice,
)
