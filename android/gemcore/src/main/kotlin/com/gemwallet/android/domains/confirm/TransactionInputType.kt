package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType

val GemTransferData.asset: Asset
    get() = inputAsset().toPrimitives()

val TransactionInputType.applicationMetadata: ApplicationMetadata?
    get() = (this as? TransactionInputType.Generic)?.metadata?.toPrimitives()

val TransactionInputType.nftAsset: NFTAsset?
    get() = (this as? TransactionInputType.TransferNft)?.nftAsset?.toPrimitives()
