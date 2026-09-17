package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import uniffi.gemstone.ApprovalData
import uniffi.gemstone.TransferDataExtra
import java.math.BigInteger

fun mockTransferDataExtra(
    to: String = "recipient",
    data: ByteArray? = null,
    outputType: TransferDataOutputType = TransferDataOutputType.EncodedTransaction,
    outputAction: TransferDataOutputAction = TransferDataOutputAction.Send,
    transactionType: TransactionType = TransactionType.Transfer,
    gasLimit: BigInteger? = null,
    approval: ApprovalData? = null,
) = TransferDataExtra(
    to = to,
    gasLimit = gasLimit,
    gasPrice = null,
    data = data,
    outputType = outputType.toGem(),
    outputAction = outputAction.toGem(),
    transactionType = transactionType.toGem(),
    approval = approval,
)
