package com.gemwallet.android.domains.confirm

import android.util.Log
import com.gemwallet.android.domains.asset.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.hex
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.toModel
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.ext.toChainType
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.StakeType
import java.nio.ByteBuffer
import java.nio.charset.CharacterCodingException
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData

fun ConfirmParams.toTransferData(): GemTransferData = GemTransferData(
    inputType = toDto(),
    recipient = GemRecipient(
        address = destination()?.address.orEmpty(),
        name = destination()?.name,
        memo = memo(),
        references = references,
    ),
    value = amount.toString(),
    useMaxAmount = useMaxAmount,
    minimumValue = minimumAmount?.toString(),
)

fun ConfirmParams.toConfirmInput(): GemConfirmInput = GemConfirmInput(
    from = from.toGem(),
    transfer = toTransferData(),
)

fun GemConfirmInput.toConfirmParams(): ConfirmParams? {
    val from = from.toPrimitives() ?: return null
    val value = transfer.value.toBigIntegerOrNull() ?: return null
    val recipient = transfer.recipient
    val destination = DestinationAddress(recipient.address, recipient.name)
    val inputType = transfer.inputType
    val asset = inputType.asset().toPrimitives() ?: return null
    val builder = ConfirmParams.Builder(asset, from, value, transfer.useMaxAmount)
    return when (inputType) {
        is GemTransactionInputType.Transfer -> builder.transfer(destination, recipient.memo, recipient.references)
        is GemTransactionInputType.Deposit -> builder.deposit(destination, recipient.memo, recipient.references)
        is GemTransactionInputType.Withdrawal -> builder.withdrawal(destination, recipient.memo, recipient.references)
        is GemTransactionInputType.Generic -> {
            val extra = inputType.extra
            ConfirmParams.TransferParams.Generic(
                asset = asset,
                from = from,
                amount = value,
                destination = destination,
                memo = recipient.memo,
                useMaxAmount = transfer.useMaxAmount,
                outputType = extra.outputType.decodeJson(),
                outputAction = extra.outputAction.decodeJson(),
                metadata = inputType.metadata.decodeJson(),
                data = extra.data.toGenericData(),
                gasLimit = extra.gasLimit,
                decodedTransactionType = extra.transactionType.decodeJson(),
                approval = extra.approval?.toModel(),
            )
        }
        is GemTransactionInputType.Swap -> ConfirmParams.SwapParams(
            from = from,
            fromAsset = asset,
            toAsset = inputType.toAsset.toPrimitives() ?: return null,
            swapData = inputType.swapData.decodeJson(),
            amount = value,
            useMaxAmount = transfer.useMaxAmount,
        )
        is GemTransactionInputType.Stake -> when (val stakeType = inputType.stakeType.decodeJson<StakeType>()) {
            is StakeType.Stake -> builder.delegate(stakeType.content)
            is StakeType.Unstake -> builder.undelegate(stakeType.content)
            is StakeType.Redelegate -> builder.redelegate(stakeType.content.toValidator, stakeType.content.delegation)
            is StakeType.Rewards -> builder.rewards(stakeType.content)
            is StakeType.Withdraw -> builder.withdraw(stakeType.content)
            is StakeType.Freeze -> builder.freeze(stakeType.content)
            is StakeType.Unfreeze -> builder.unfreeze(stakeType.content)
        }
        is GemTransactionInputType.TransferNft -> ConfirmParams.NftParams(
            asset = asset,
            from = from,
            destination = destination,
            nftAsset = inputType.nftAsset.decodeJson(),
        )
        is GemTransactionInputType.Account -> builder.activate(inputType.accountType.decodeJson())
        is GemTransactionInputType.Perpetual -> builder.perpetual(inputType.perpetualType.decodeJson())
        is GemTransactionInputType.TokenApprove, is GemTransactionInputType.Earn -> {
            Log.e("ConfirmInput", "no confirm params for ${inputType::class.simpleName}")
            null
        }
    }
}

private fun GemTransactionInputType.asset(): String = when (this) {
    is GemTransactionInputType.Transfer -> asset
    is GemTransactionInputType.Deposit -> asset
    is GemTransactionInputType.Withdrawal -> asset
    is GemTransactionInputType.Generic -> asset
    is GemTransactionInputType.Swap -> fromAsset
    is GemTransactionInputType.Stake -> asset
    is GemTransactionInputType.TransferNft -> asset
    is GemTransactionInputType.Account -> asset
    is GemTransactionInputType.Perpetual -> asset
    is GemTransactionInputType.TokenApprove -> asset
    is GemTransactionInputType.Earn -> asset
}

private fun ByteArray?.toGenericData(): String {
    this ?: return ""
    return try {
        Charsets.UTF_8.newDecoder().decode(ByteBuffer.wrap(this)).toString()
    } catch (_: CharacterCodingException) {
        "0x$hex"
    }
}

fun GemTransferData.toGenericParams(account: Account): ConfirmParams.TransferParams.Generic {
    val input = inputType as? GemTransactionInputType.Generic ?: throw IllegalArgumentException("WalletConnect transfer is not generic")
    val asset = input.asset.decodeJson<Asset>()
    val data = input.extra.data ?: ByteArray(0)
    return ConfirmParams.TransferParams.Generic(
        asset = asset,
        from = account,
        amount = value.toBigInteger(),
        destination = DestinationAddress(input.extra.to),
        outputType = input.extra.outputType.decodeJson(),
        outputAction = input.extra.outputAction.decodeJson(),
        metadata = input.metadata.decodeJson(),
        data = if (asset.id.chain.toChainType() == ChainType.Ethereum) "0x${data.hex}" else String(data),
        gasLimit = input.extra.gasLimit,
        decodedTransactionType = input.extra.transactionType.decodeJson(),
        approval = input.extra.approval?.decodeJson(),
    )
}
