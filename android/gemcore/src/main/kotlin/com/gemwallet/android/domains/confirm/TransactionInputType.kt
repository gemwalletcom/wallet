package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.toGem
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AccountDataType
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.StakeType
import com.wallet.core.primitives.swap.SwapData
import uniffi.gemstone.TransactionInputType

val TransactionInputType.asset: Asset
    get() = when (this) {
        is TransactionInputType.Transfer -> asset
        is TransactionInputType.Deposit -> asset
        is TransactionInputType.Withdrawal -> asset
        is TransactionInputType.Stake -> asset
        is TransactionInputType.TokenApprove -> asset
        is TransactionInputType.Account -> asset
        is TransactionInputType.Perpetual -> asset
        is TransactionInputType.TransferNft -> asset
        is TransactionInputType.Generic -> asset
        is TransactionInputType.Earn -> asset
        is TransactionInputType.Swap -> fromAsset
    }.toPrimitives()

val TransactionInputType.chain: Chain
    get() = asset.chain

val TransactionInputType.toAsset: Asset?
    get() = (this as? TransactionInputType.Swap)?.toAsset?.toPrimitives()

val TransactionInputType.applicationMetadata: ApplicationMetadata?
    get() = (this as? TransactionInputType.Generic)?.metadata?.toPrimitives()

val TransactionInputType.swapData: SwapData?
    get() = (this as? TransactionInputType.Swap)?.swapData?.decodeJson<SwapData>()

val TransactionInputType.nftAsset: NFTAsset?
    get() = (this as? TransactionInputType.TransferNft)?.nftAsset?.toPrimitives()

val TransactionInputType.stakeType: StakeType?
    get() = (this as? TransactionInputType.Stake)?.stakeType?.decodeJson<StakeType>()

val TransactionInputType.perpetualType: PerpetualType?
    get() = (this as? TransactionInputType.Perpetual)?.perpetualType?.decodeJson<PerpetualType>()

fun TransactionInputType.Companion.transfer(asset: Asset): TransactionInputType =
    TransactionInputType.Transfer(asset.toGem())

fun TransactionInputType.Companion.deposit(asset: Asset): TransactionInputType =
    TransactionInputType.Deposit(asset.toGem())

fun TransactionInputType.Companion.transferNft(asset: Asset, nftAsset: NFTAsset): TransactionInputType =
    TransactionInputType.TransferNft(asset.toGem(), nftAsset.toGem())

fun TransactionInputType.Companion.swap(fromAsset: Asset, toAsset: Asset, swapData: SwapData): TransactionInputType =
    TransactionInputType.Swap(fromAsset.toGem(), toAsset.toGem(), swapData.toJson())

fun TransactionInputType.Companion.account(asset: Asset, accountType: AccountDataType): TransactionInputType =
    TransactionInputType.Account(asset.toGem(), accountType.toGem())
