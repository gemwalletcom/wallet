package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemPaymentTransaction
import uniffi.gemstone.PaymentRequest

fun mockGemPaymentTransaction(
    account: Account = mockAccount(),
    request: PaymentRequest = mockPaymentRequest(address = account.address),
) = GemPaymentTransaction(
    merchant = mockApplicationMetadata(source = ApplicationMetadataSource.Payment).toGem(),
    account = ChainAddress(account.chain, account.address).toGem(),
    transaction = "encoded-transaction",
    transactionType = TransactionType.Transfer.toGem(),
    memo = request.memo,
    request = request,
)
