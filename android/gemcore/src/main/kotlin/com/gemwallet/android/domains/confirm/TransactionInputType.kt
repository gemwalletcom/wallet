package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.SwapData
import uniffi.gemstone.TransactionInputType

val GemTransferData.asset: Asset
    get() = inputAsset().toPrimitives()

val TransactionInputType.toAsset: Asset?
    get() = (this as? TransactionInputType.Swap)?.toAsset?.toPrimitives()

val TransactionInputType.applicationMetadata: ApplicationMetadata?
    get() = (this as? TransactionInputType.Generic)?.metadata?.toPrimitives()

val TransactionInputType.swapData: SwapData?
    get() = (this as? TransactionInputType.Swap)?.swapData

val TransactionInputType.nftAsset: NFTAsset?
    get() = (this as? TransactionInputType.TransferNft)?.nftAsset?.toPrimitives()

val TransactionInputType.perpetualType: PerpetualType?
    get() = (this as? TransactionInputType.Perpetual)?.perpetualType
