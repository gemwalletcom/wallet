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
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import com.wallet.core.primitives.swap.SwapData
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferService

val GemTransactionInputType.asset: Asset
    get() = when (this) {
        is GemTransactionInputType.Transfer -> asset
        is GemTransactionInputType.Deposit -> asset
        is GemTransactionInputType.Withdrawal -> asset
        is GemTransactionInputType.Stake -> asset
        is GemTransactionInputType.TokenApprove -> asset
        is GemTransactionInputType.Account -> asset
        is GemTransactionInputType.Perpetual -> asset
        is GemTransactionInputType.TransferNft -> asset
        is GemTransactionInputType.Generic -> asset
        is GemTransactionInputType.Earn -> asset
        is GemTransactionInputType.Swap -> fromAsset
    }.toPrimitives()

val GemTransactionInputType.chain: Chain
    get() = asset.chain

val GemTransactionInputType.toAsset: Asset?
    get() = (this as? GemTransactionInputType.Swap)?.toAsset?.toPrimitives()

val GemTransactionInputType.applicationMetadata: ApplicationMetadata?
    get() = (this as? GemTransactionInputType.Generic)?.metadata?.decodeJson<ApplicationMetadata>()

val GemTransactionInputType.swapData: SwapData?
    get() = (this as? GemTransactionInputType.Swap)?.swapData?.decodeJson<SwapData>()

val GemTransactionInputType.nftAsset: NFTAsset?
    get() = (this as? GemTransactionInputType.TransferNft)?.nftAsset?.decodeJson<NFTAsset>()

val GemTransactionInputType.stakeType: StakeType?
    get() = (this as? GemTransactionInputType.Stake)?.stakeType?.decodeJson<StakeType>()

val GemTransactionInputType.perpetualType: PerpetualType?
    get() = (this as? GemTransactionInputType.Perpetual)?.perpetualType?.decodeJson<PerpetualType>()

fun GemTransactionInputType.transactionType(transferService: GemTransferService): TransactionType =
    transferService.transactionType(this).decodeJson<TransactionType>()

fun GemTransactionInputType.approvalData(
    transactionType: TransactionType,
    transferService: GemTransferService,
): ApprovalData? = transferService.approval(this, transactionType.toJson())?.decodeJson<ApprovalData>()

fun GemTransactionInputType.Companion.transfer(asset: Asset): GemTransactionInputType =
    GemTransactionInputType.Transfer(asset.toGem())

fun GemTransactionInputType.Companion.deposit(asset: Asset): GemTransactionInputType =
    GemTransactionInputType.Deposit(asset.toGem())

fun GemTransactionInputType.Companion.withdrawal(asset: Asset): GemTransactionInputType =
    GemTransactionInputType.Withdrawal(asset.toGem())

fun GemTransactionInputType.Companion.transferNft(asset: Asset, nftAsset: NFTAsset): GemTransactionInputType =
    GemTransactionInputType.TransferNft(asset.toGem(), nftAsset.toGem())

fun GemTransactionInputType.Companion.swap(fromAsset: Asset, toAsset: Asset, swapData: SwapData): GemTransactionInputType =
    GemTransactionInputType.Swap(fromAsset.toGem(), toAsset.toGem(), swapData.toJson())

fun GemTransactionInputType.Companion.stake(asset: Asset, stakeType: StakeType): GemTransactionInputType =
    GemTransactionInputType.Stake(asset.toGem(), stakeType.toJson())

fun GemTransactionInputType.Companion.account(asset: Asset, accountType: AccountDataType): GemTransactionInputType =
    GemTransactionInputType.Account(asset.toGem(), accountType.toJson())

fun GemTransactionInputType.Companion.perpetual(asset: Asset, perpetualType: PerpetualType): GemTransactionInputType =
    GemTransactionInputType.Perpetual(asset.toGem(), perpetualType.toGem())
