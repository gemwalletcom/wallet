package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.toGem
import com.gemwallet.android.domains.confirm.toConfirmInput
import com.gemwallet.android.domains.confirm.toConfirmParams
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.domains.stake.toGem
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.type
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.serializer.unpackRouteString
import com.wallet.core.primitives.Account
import android.util.Log
import com.wallet.core.primitives.AccountDataType
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.RedelegateData
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.StakeType
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction as PrimitiveOutputAction
import com.wallet.core.primitives.TransferDataOutputType as PrimitiveOutputType
import com.wallet.core.primitives.swap.ApprovalData
import com.wallet.core.primitives.swap.SwapData
import java.math.BigInteger
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransactionInputType.*
import uniffi.gemstone.GemTransferDataExtra
import uniffi.gemstone.SwapperProvider

import uniffi.gemstone.GemTransferService
private val transferService = GemTransferService()

sealed class ConfirmParams() {

    abstract val asset: Asset

    abstract val from: Account
    abstract val amount: BigInteger

    abstract val useMaxAmount: Boolean

    open val minimumAmount: BigInteger?
        get() = null

    open val references: List<String>
        get() = emptyList()

    val assetId: AssetId get() = asset.id

    class Builder(
        val asset: Asset,
        val from: Account,
        val amount: BigInteger = BigInteger.ZERO,
        val useMaxAmount: Boolean = false,
    ) {
        fun transfer(destination: DestinationAddress, memo: String? = null, references: List<String> = emptyList()): TransferParams {
            return when (asset.id.type()) {
                AssetSubtype.NATIVE -> TransferParams.Native(
                    asset = asset,
                    from = from,
                    amount = amount,
                    destination = destination,
                    memo = memo,
                    references = references,
                    useMaxAmount = useMaxAmount
                )
                AssetSubtype.TOKEN -> TransferParams.Token(
                    asset = asset,
                    from = from,
                    amount = amount,
                    destination = destination,
                    memo = memo,
                    references = references,
                    useMaxAmount = useMaxAmount
                )
            }
        }

        fun deposit(
            destination: DestinationAddress,
            memo: String? = null,
            references: List<String> = emptyList(),
        ): TransferParams.Deposit = TransferParams.Deposit(
            asset = asset,
            from = from,
            amount = amount,
            destination = destination,
            memo = memo,
            references = references,
            useMaxAmount = useMaxAmount,
        )

        fun withdrawal(
            destination: DestinationAddress,
            memo: String? = null,
            references: List<String> = emptyList(),
        ): TransferParams.Withdrawal = TransferParams.Withdrawal(
            asset = asset,
            from = from,
            amount = amount,
            destination = destination,
            memo = memo,
            references = references,
            useMaxAmount = useMaxAmount,
        )

        fun delegate(validator: DelegationValidator) = Stake.DelegateParams(asset, from, amount, validator, useMaxAmount)

        fun rewards(validators: List<DelegationValidator>) = Stake.RewardsParams(asset, from, validators, amount)

        fun withdraw(delegation: Delegation) = Stake.WithdrawParams(
            asset = asset,
            from = from,
            amount = amount,
            delegation = delegation,
        )

        fun undelegate(delegation: Delegation): Stake.UndelegateParams {
            return Stake.UndelegateParams(
                asset,
                from,
                amount,
                delegation,
            )
        }

        fun redelegate(destinationValidator: DelegationValidator, delegation: Delegation): Stake.RedelegateParams {
            return Stake.RedelegateParams(
                asset,
                from = from,
                amount,
                delegation,
                destinationValidator,
            )
        }

        fun activate(accountType: AccountDataType = AccountDataType.Activate): Activate {
            return Activate(asset, from, accountType = accountType)
        }

        fun freeze(resource: Resource): Stake.Freeze {
            return Stake.Freeze(asset, from, amount, resource, useMaxAmount)
        }

        fun unfreeze(resource: Resource): Stake.Unfreeze {
            return Stake.Unfreeze(asset, from, amount, resource)
        }

        fun perpetual(perpetualType: PerpetualType): PerpetualParams =
            PerpetualParams(asset, from, amount, useMaxAmount, perpetualType)
    }

    abstract fun toDto(): GemTransactionInputType
    sealed class TransferParams : ConfirmParams() {
        abstract val destination: DestinationAddress
        abstract val memo: String?

        override fun destination(): DestinationAddress {
            return destination
        }

        override fun memo(): String? {
            return memo
        }
        class Generic(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger = BigInteger.ZERO,
            override val destination: DestinationAddress = DestinationAddress(""),
            override val memo: String? = null,
            override val useMaxAmount: Boolean = false,
            val outputType: PrimitiveOutputType = PrimitiveOutputType.EncodedTransaction,
            val outputAction: PrimitiveOutputAction = PrimitiveOutputAction.Send,
            val metadata: ApplicationMetadata,
            val data: String,
            val gasLimit: String?,
            val decodedTransactionType: TransactionType = TransactionType.SmartContractCall,
            val approval: ApprovalData? = null,
        ) : TransferParams() {
            val isSendable: Boolean
                get() = outputAction == PrimitiveOutputAction.Send

            override fun toDto(): GemTransactionInputType {
                return Generic(
                    asset = asset.toGem(),
                    metadata = metadata.toJson(),
                    extra = GemTransferDataExtra(
                        gasLimit = gasLimit,
                        gasPrice = null,
                        data = data.let { data ->
                            if (data.has0xPrefix()) {
                                try {
                                    return@let data.fromHex()
                                } catch (_: IllegalArgumentException) { }
                            }
                            data.toByteArray()
                        },
                        outputType = outputType.toJson(),
                        outputAction = outputAction.toJson(),
                        transactionType = decodedTransactionType.toJson(),
                        to = destination().address,
                        approval = approval?.toJson(),
                    ),
                )
            }

            override fun hashCode(): Int {
                var result = asset.hashCode()
                result = 31 * result + from.hashCode()
                result = 31 * result + amount.hashCode()
                result = 31 * result + destination.hashCode()
                result = 31 * result + memo.hashCode()
                result = 31 * result + useMaxAmount.hashCode()
                result = 31 * result + metadata.hashCode()
                result = 31 * result + data.hashCode()
                result = 31 * result + (gasLimit?.hashCode() ?: 0)
                result = 31 * result + decodedTransactionType.hashCode()
                result = 31 * result + (approval?.hashCode() ?: 0)
                return result
            }

        }
        class Native(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val useMaxAmount: Boolean = false,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Transfer(asset.toGem())
        }
        class Token(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val useMaxAmount: Boolean = false,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = Transfer(asset.toGem())
        }
        class Deposit(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val useMaxAmount: Boolean = false,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Deposit(asset.toGem())
        }
        class Withdrawal(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val useMaxAmount: Boolean = false,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Withdrawal(asset.toGem())
        }
    }
    class SwapParams(
        override val from: Account,
        val fromAsset: Asset,
        val toAsset: Asset,
        val swapData: SwapData,
        override val amount: BigInteger,
        override val useMaxAmount: Boolean = false,
    ) : ConfirmParams() {

        override val asset: Asset
            get() = fromAsset

        override val minimumAmount: BigInteger?
            get() = swapData.quote.minFromValue?.toBigIntegerOrNull()

        val toAmount: BigInteger
            get() = swapData.quote.toValue.toBigInteger()

        val providerId: SwapperProvider
            get() = SwapperProvider.entries.first { it.name.lowercase() == swapData.quote.providerData.provider.string }

        val protocol: String
            get() = swapData.quote.providerData.protocolName

        val slippageBps: UInt
            get() = swapData.quote.slippageBps

        val etaInSeconds: UInt?
            get() = swapData.quote.etaInSeconds

        val approval: ApprovalData?
            get() = swapData.data.approval

        override fun toDto(): GemTransactionInputType = Swap(
            fromAsset = fromAsset.toGem(),
            toAsset = toAsset.toGem(),
            swapData = swapData.toJson(),
        )

        override fun destination(): DestinationAddress = DestinationAddress(swapData.data.to)

        override fun memo(): String? = swapData.data.memo
    }
    class Activate(
        override val asset: Asset,
        override val from: Account,
        override val amount: BigInteger = BigInteger.ZERO,
        val accountType: AccountDataType = AccountDataType.Activate,
    ) : ConfirmParams() {
        override val useMaxAmount: Boolean
            get() = false

        override fun toDto(): GemTransactionInputType =
            Account(asset.toGem(), accountType.toJson())

        override fun destination(): DestinationAddress {
            return DestinationAddress(from.address)
        }
    }
    class NftParams(
        override val asset: Asset,
        override val from: Account,
        val destination: DestinationAddress,
        val nftAsset: NFTAsset,
    ) : ConfirmParams() {
        override val useMaxAmount: Boolean
            get() = false

        override fun toDto(): GemTransactionInputType = TransferNft(
                asset.toGem(),
                nftAsset.toGem(),
            )

        override val amount: BigInteger = BigInteger.ZERO

        override fun destination(): DestinationAddress {
            return destination
        }
    }
    sealed class Stake : ConfirmParams() {
        class DelegateParams(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val validator: DelegationValidator,
            override val useMaxAmount: Boolean = false,
        ) : Stake() {

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Stake(validator) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress(validator.id)
            }
        }
        class WithdrawParams(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val delegation: Delegation,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Withdraw(delegation) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress(delegation.validator.id)
            }
        }
        class UndelegateParams(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val delegation: Delegation,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Unstake(delegation) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress(delegation.validator.id)
            }
        }
        class RedelegateParams(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val delegation: Delegation,
            val destinationValidator: DelegationValidator,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Redelegate(RedelegateData(delegation, destinationValidator)) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress("")
            }
        }
        class RewardsParams(
            override val asset: Asset,
            override val from: Account,
            val validators: List<DelegationValidator>,
            override val amount: BigInteger,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Rewards(validators) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress("")
            }
        }
        class Freeze(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val resource: Resource,
            override val useMaxAmount: Boolean = false,
        ) : Stake() {

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Freeze(resource) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress("")
            }
        }
        class Unfreeze(
            override val asset: Asset,
            override val from: Account,
            override val amount: BigInteger,
            val resource: Resource,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = (StakeType.Unfreeze(resource) as StakeType).toJson()
            )

            override fun destination(): DestinationAddress {
                return DestinationAddress("")
            }
        }
    }
    data class PerpetualParams(
        override val asset: Asset,
        override val from: Account,
        override val amount: BigInteger,
        override val useMaxAmount: Boolean = false,
        val perpetualType: PerpetualType,
    ) : ConfirmParams() {

        override fun destination(): DestinationAddress = DestinationAddress.hyperliquidProvider

        override fun toDto(): GemTransactionInputType = GemTransactionInputType.Perpetual(
            asset = asset.toGem(),
            perpetualType = perpetualType.toGem(),
        )
    }

    fun pack(): String? = runCatching { transferService.encodeConfirmInput(toConfirmInput()).packRouteString() }
        .onFailure { Log.e(TAG, "confirm params encode failed", it) }
        .getOrNull()

    fun getTransactionType(): TransactionType = transferService.transactionType(toDto()).decodeJson<TransactionType>()

    open fun destination(): DestinationAddress? = null

    open fun memo(): String? = null

    override fun hashCode(): Int {
        return asset.id.toIdentifier().hashCode() +
                destination().hashCode() +
                memo().hashCode() +
                references.hashCode() +
                amount.hashCode() +
                useMaxAmount.hashCode()
    }

    companion object {
        private const val TAG = "ConfirmParams"

        fun unpack(input: String): ConfirmParams? = runCatching {
            transferService.decodeConfirmInput(input.unpackRouteString()).toConfirmParams()
        }
            .onFailure { Log.e(TAG, "confirm params decode failed", it) }
            .getOrNull()
    }
}

fun uniffi.gemstone.ApprovalData.toModel(): ApprovalData = decodeJson<ApprovalData>()
