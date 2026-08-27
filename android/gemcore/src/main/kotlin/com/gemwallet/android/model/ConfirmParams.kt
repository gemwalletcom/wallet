package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.toGem
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.toConfirmInput
import com.gemwallet.android.domains.confirm.toConfirmParams
import com.gemwallet.android.domains.confirm.toGem
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.domains.stake.toGem
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.type
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.BigIntegerSerializer
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.serializer.unpackRoutePayload
import com.gemwallet.android.serializer.unpackRouteString
import com.wallet.core.primitives.Account
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
import com.wallet.core.primitives.swap.SwapQuoteDataType
import java.math.BigInteger
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransactionInputType.*
import uniffi.gemstone.GemTransferDataExtra
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.confirmInputDecode
import uniffi.gemstone.confirmInputEncode
import uniffi.gemstone.GemstoneException
import uniffi.gemstone.GemTransferService

@Serializable
sealed class ConfirmParams() {

    abstract val asset: Asset

    abstract val from: Account

    @Serializable(BigIntegerSerializer::class)
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

        fun deposit(destination: DestinationAddress): TransferParams.Deposit = TransferParams.Deposit(
            asset = asset,
            from = from,
            amount = amount,
            destination = destination,
            useMaxAmount = useMaxAmount,
        )

        fun withdrawal(destination: DestinationAddress): TransferParams.Withdrawal = TransferParams.Withdrawal(
            asset = asset,
            from = from,
            amount = amount,
            destination = destination,
            useMaxAmount = useMaxAmount,
        )

        fun approval(approvalData: String, provider: String, contract: String = ""): TokenApprovalParams {
            return TokenApprovalParams(asset, from, approvalData, provider, contract)
        }

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

        fun activate(): Activate {
            return Activate(asset, from)
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

    @Serializable
    sealed class TransferParams : ConfirmParams() {
        abstract val destination: DestinationAddress
        abstract val memo: String?
        abstract val inputType: InputType?

        override fun destination(): DestinationAddress {
            return destination
        }

        override fun memo(): String? {
            return memo
        }

        @Serializable
        class Generic(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger = BigInteger.ZERO,
            override val destination: DestinationAddress = DestinationAddress(""),
            override val memo: String? = null,
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType,
            val isSendable: Boolean,
            val metadata: ApplicationMetadata,
            val data: String,
            val gasLimit: String?,
            val decodedTransactionType: TransactionType = TransactionType.SmartContractCall,
            val approval: ApprovalData? = null,
        ) : TransferParams() {
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
                        outputType = when (inputType) {
                            InputType.Signature -> PrimitiveOutputType.Signature
                            InputType.EncodeTransaction -> PrimitiveOutputType.EncodedTransaction
                        }.toJson(),
                        outputAction = when (inputType) {
                            InputType.Signature -> PrimitiveOutputAction.Sign
                            InputType.EncodeTransaction -> PrimitiveOutputAction.Send
                        }.toJson(),
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

        @Serializable
        class Native(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val inputType: InputType? = null,
            override val useMaxAmount: Boolean = false,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Transfer(asset.toGem())
        }

        @Serializable
        class Token(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val references: List<String> = emptyList(),
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType? = null,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = Transfer(asset.toGem())
        }

        @Serializable
        class Deposit(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType? = null,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Deposit(asset.toGem())
        }

        @Serializable
        class Withdrawal(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
            override val destination: DestinationAddress,
            override val memo: String? = null,
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType? = null,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = GemTransactionInputType.Withdrawal(asset.toGem())
        }

        @Serializable
        enum class InputType {
            Signature,
            EncodeTransaction,
        }
    }

    @Serializable
    class TokenApprovalParams(
        override val asset: Asset,
        override val from: Account,
        val data: String,
        val provider: String,
        val contract: String,
    ) : ConfirmParams() {
        override val useMaxAmount: Boolean = false

        val approval: ApprovalData
            get() = ApprovalData(
                token = requireNotNull(asset.id.tokenId),
                spender = contract,
                value = amount.toString(),
                isUnlimited = true,
            )

        override fun toDto(): GemTransactionInputType = TokenApprove(asset.toGem(), approval.toJson())

        override val amount: BigInteger
            get() = BigInteger.ZERO

        override fun memo(): String = data

        override fun destination(): DestinationAddress {
            return DestinationAddress(contract)
        }
    }

    @Serializable
    class SwapParams(
        override val from: Account,
        val fromAsset: Asset,
        @Serializable(BigIntegerSerializer::class) val fromAmount: BigInteger,
        @Serializable(BigIntegerSerializer::class) val minFromAmount: BigInteger? = null,
        val toAsset: Asset,
        @Serializable(BigIntegerSerializer::class) val toAmount: BigInteger,
        val swapData: String,
        val memo: String?,
        val providerId: SwapperProvider,
        val providerName: String,
        val protocol: String,
        val protocolId: String,
        val toAddress: String,
        val value: String,
        val approval: ApprovalData? = null,
        val slippageBps: UInt,
        val etaInSeconds: UInt?,
        val dataType: SwapQuoteDataType,
        @Serializable(BigIntegerSerializer::class) val gasLimit: BigInteger? = null,
        override val useMaxAmount: Boolean = false,
    ) : ConfirmParams() {

        override val asset: Asset
            get() = fromAsset

        override val amount: BigInteger
            get() = fromAmount

        override val minimumAmount: BigInteger?
            get() = minFromAmount

        override fun toDto(): GemTransactionInputType = Swap(
            fromAsset = fromAsset.toGem(),
            toAsset = toAsset.toGem(),
            swapData = toGem(),
        )

        override fun destination(): DestinationAddress = DestinationAddress(toAddress)

        override fun memo(): String? = memo

    }

    @Serializable
    class Activate(
        override val asset: Asset,
        override val from: Account,
        @Serializable(BigIntegerSerializer::class) override val amount: BigInteger = BigInteger.ZERO,
    ) : ConfirmParams() {
        override val useMaxAmount: Boolean
            get() = false

        override fun toDto(): GemTransactionInputType =
            Account(asset.toGem(), AccountDataType.Activate.toJson())

        override fun destination(): DestinationAddress {
            return DestinationAddress(from.address)
        }
    }

    @Serializable
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

        @Serializable(BigIntegerSerializer::class) override val amount: BigInteger = BigInteger.ZERO

        override fun destination(): DestinationAddress {
            return destination
        }
    }

    @Serializable
    sealed class Stake : ConfirmParams() {

        @Serializable
        class DelegateParams(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class WithdrawParams(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class UndelegateParams(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class RedelegateParams(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class RewardsParams(
            override val asset: Asset,
            override val from: Account,
            val validators: List<DelegationValidator>,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class Freeze(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

        @Serializable
        class Unfreeze(
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
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

    @Serializable
    data class PerpetualParams(
        override val asset: Asset,
        override val from: Account,
        @Serializable(BigIntegerSerializer::class) override val amount: BigInteger,
        override val useMaxAmount: Boolean = false,
        val perpetualType: PerpetualType,
    ) : ConfirmParams() {

        override fun destination(): DestinationAddress = DestinationAddress.hyperliquidProvider

        override fun toDto(): GemTransactionInputType = GemTransactionInputType.Perpetual(
            asset = asset.toGem(),
            perpetualType = perpetualType.toGem(),
        )
    }

    fun approvalData(transactionType: TransactionType): ApprovalData? = try {
        GemTransferService().approval(toDto(), transactionType.toJson())?.decodeJson<ApprovalData>()
    } catch (_: GemstoneException) {
        throw ConfirmError.TransactionIncorrect
    }

    fun pack(): String? = when (this) {
        is TransferParams.Native, is TransferParams.Token, is TransferParams.Generic ->
            runCatching { confirmInputEncode(toConfirmInput()).packRouteString() }.getOrNull() ?: packRoutePayload()
        else -> packRoutePayload()
    }

    fun getTransactionType(): TransactionType = GemTransferService().transactionType(toDto()).decodeJson<TransactionType>()

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
        fun unpack(input: String): ConfirmParams? = runCatching {
            confirmInputDecode(input.unpackRouteString()).toConfirmParams()
        }.getOrNull() ?: unpackRoutePayload(input)
    }
}

fun uniffi.gemstone.ApprovalData.toModel(): ApprovalData = decodeJson<ApprovalData>()
