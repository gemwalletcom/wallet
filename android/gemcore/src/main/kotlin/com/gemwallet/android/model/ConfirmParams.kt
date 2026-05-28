package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.toGem
import com.gemwallet.android.domains.confirm.toGem
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.domains.stake.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.type
import com.gemwallet.android.ext.urlDecode
import com.gemwallet.android.ext.urlEncode
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.BigIntegerSerializer
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import kotlinx.serialization.Serializable
import org.json.JSONObject
import uniffi.gemstone.GemAccountDataType
import uniffi.gemstone.GemApprovalData
import uniffi.gemstone.GemResource
import uniffi.gemstone.GemStakeType
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransactionInputType.*
import uniffi.gemstone.GemTransferDataExtra
import uniffi.gemstone.GemWalletConnectionSessionAppMetadata
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.TransferDataOutputAction
import uniffi.gemstone.TransferDataOutputType
import java.math.BigInteger
import java.util.Base64

@Serializable
sealed class ConfirmParams() {

    abstract val asset: Asset

    abstract val from: Account

    @Serializable(BigIntegerSerializer::class)
    abstract val amount: BigInteger

    abstract val useMaxAmount: Boolean

    abstract val shouldIgnoreValueCheck: Boolean

    open val minimumAmount: BigInteger?
        get() = null

    val assetId: AssetId get() = asset.id

    class Builder(
        val asset: Asset,
        val from: Account,
        val amount: BigInteger = BigInteger.ZERO,
        val useMaxAmount: Boolean = false,
    ) {
        fun transfer(destination: DestinationAddress, memo: String? = null): TransferParams {
            return when (asset.id.type()) {
                AssetSubtype.NATIVE -> TransferParams.Native(
                    asset = asset,
                    from = from,
                    amount = amount,
                    destination = destination,
                    memo = memo,
                    useMaxAmount = useMaxAmount
                )
                AssetSubtype.TOKEN -> TransferParams.Token(
                    asset = asset,
                    from = from,
                    amount = amount,
                    destination = destination,
                    memo = memo,
                    useMaxAmount = useMaxAmount
                )
            }
        }

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
                delegation.base.shares,
                delegation.base.balance,
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

        override val shouldIgnoreValueCheck: Boolean
            get() = false

        override fun destination(): DestinationAddress {
            return destination
        }

        override fun memo(): String? {
            return memo
        }

        @Serializable
        class Generic(
            val requestId: String,
            override val asset: Asset,
            override val from: Account,
            @Serializable(BigIntegerSerializer::class) override val amount: BigInteger = BigInteger.ZERO,
            override val destination: DestinationAddress = DestinationAddress(""),
            override val memo: String? = null,
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType? = null,
            val isSendable: Boolean,
            val name: String,
            val description: String,
            val url: String,
            val icon: String,
            val gasLimit: String?,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType {
                val type = requireNotNull(inputType) { "inputType is required for Generic transactions" }
                return Generic(
                asset = asset.toGem(),
                metadata = GemWalletConnectionSessionAppMetadata(
                    name = name,
                    description = description,
                    url = url,
                    icon = icon,
                ),
                extra = GemTransferDataExtra(
                    gasLimit = null,
                    gasPrice = null,
                    data = memo?.let { data ->
                        if (data.has0xPrefix()) {
                            try {
                                return@let data.fromHex()
                            } catch (_: Error) { }
                        }
                        data.toByteArray()
                    },
                    outputType = when (type) {
                        InputType.Signature -> TransferDataOutputType.SIGNATURE
                        InputType.EncodeTransaction -> TransferDataOutputType.ENCODED_TRANSACTION
                    },
                    outputAction = when (type) {
                        InputType.Signature -> TransferDataOutputAction.SIGN
                        InputType.EncodeTransaction -> TransferDataOutputAction.SEND
                    },
                    to = destination().address
                ),
            )
            }

            override fun hashCode(): Int {
                var result = asset.hashCode()
                result = 31 * result + requestId.hashCode()
                result = 31 * result + from.hashCode()
                result = 31 * result + amount.hashCode()
                result = 31 * result + destination.hashCode()
                result = 31 * result + memo.hashCode()
                result = 31 * result + useMaxAmount.hashCode()
                result = 31 * result + name.hashCode()
                result = 31 * result + destination.hashCode()
                result = 31 * result + url.hashCode()
                result = 31 * result + icon.hashCode()
                result = 31 * result + (gasLimit?.hashCode() ?: 0)
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
            override val useMaxAmount: Boolean = false,
            override val inputType: InputType? = null,
        ) : TransferParams() {
            override fun toDto(): GemTransactionInputType = Transfer(asset.toGem())

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

        override val shouldIgnoreValueCheck: Boolean
            get() = false

        override fun toDto(): GemTransactionInputType = TokenApprove(
            asset.toGem(),
            GemApprovalData(
                assetId.tokenId!!,
                spender = contract,
                value = amount.toString(),
                isUnlimited = true,
            )
        )

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
        val dataType: GemSwapQuoteDataType,
        @Serializable(BigIntegerSerializer::class) val gasLimit: BigInteger? = null,
        override val useMaxAmount: Boolean = false,
    ) : ConfirmParams() {

        override val asset: Asset
            get() = fromAsset

        override val amount: BigInteger
            get() = fromAmount

        override val minimumAmount: BigInteger?
            get() = minFromAmount

        override val shouldIgnoreValueCheck: Boolean
            get() = false

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

        override val shouldIgnoreValueCheck: Boolean
            get() = false

        override fun toDto(): GemTransactionInputType =
            Account(asset.toGem(), GemAccountDataType.ACTIVATE)

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

        override val shouldIgnoreValueCheck: Boolean
            get() = true

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
            override val shouldIgnoreValueCheck: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Delegate(validator.toGem(asset.chain.string))
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

            override val shouldIgnoreValueCheck: Boolean
                get() = true

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Withdraw(delegation.toGem(asset.chain.string))
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

            override val shouldIgnoreValueCheck: Boolean
                get() = true

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Undelegate(
                    delegation = delegation.toGem(asset.chain.string),
                )
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
            val share: String?,
            val balance: String?,
        ) : Stake() {
            override val useMaxAmount: Boolean
                get() = false

            override val shouldIgnoreValueCheck: Boolean
                get() = true

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Redelegate(
                    delegation = delegation.toGem(asset.chain.string),
                    toValidator = destinationValidator.toGem(asset.chain.string)
                )
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

            override val shouldIgnoreValueCheck: Boolean
                get() = true

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.WithdrawRewards(
                    validators = validators.map { it.toGem(asset.chain.string) }
                )
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
            override val shouldIgnoreValueCheck: Boolean
                get() = false

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Freeze(
                    resource = when (resource) {
                        Resource.Energy -> GemResource.ENERGY
                        Resource.Bandwidth -> GemResource.BANDWIDTH
                    }
                )
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

            override val shouldIgnoreValueCheck: Boolean
                get() = true

            override fun toDto(): GemTransactionInputType = Stake(
                asset = asset.toGem(),
                stakeType = GemStakeType.Unfreeze(
                    resource = when (resource) {
                        Resource.Energy -> GemResource.ENERGY
                        Resource.Bandwidth -> GemResource.BANDWIDTH
                    }
                )
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

        override val shouldIgnoreValueCheck: Boolean
            get() = true

        override fun destination(): DestinationAddress = DestinationAddress.Hyperliquid

        override fun toDto(): GemTransactionInputType = GemTransactionInputType.Perpetual(
            asset = asset.toGem(),
            perpetualType = perpetualType.toGem(),
        )
    }

    fun pack(): String? {
        val json = jsonEncoder.encodeToString(this)
        return Base64.getEncoder().encodeToString(json.toByteArray()).urlEncode()
    }

    fun getTransactionType() : TransactionType {
        return when (this) {
            is TransferParams -> TransactionType.Transfer
            is TokenApprovalParams -> TransactionType.TokenApproval
            is SwapParams -> TransactionType.Swap
            is Activate -> TransactionType.AssetActivation
            is NftParams -> TransactionType.TransferNFT
            is Stake.DelegateParams -> TransactionType.StakeDelegate
            is Stake.RewardsParams -> TransactionType.StakeRewards
            is Stake.RedelegateParams -> TransactionType.StakeRedelegate
            is Stake.UndelegateParams -> TransactionType.StakeUndelegate
            is Stake.WithdrawParams -> TransactionType.StakeWithdraw
            is Stake.Freeze -> TransactionType.StakeFreeze
            is Stake.Unfreeze -> TransactionType.StakeUnfreeze
            is Stake -> throw IllegalArgumentException("Invalid stake parameter")
            is PerpetualParams -> when (perpetualType) {
                is PerpetualType.Open -> TransactionType.PerpetualOpenPosition
                is PerpetualType.Close -> TransactionType.PerpetualClosePosition
                is PerpetualType.Increase,
                is PerpetualType.Reduce,
                is PerpetualType.Modify -> TransactionType.PerpetualModifyPosition
            }
        }
    }

    open  fun destination(): DestinationAddress? = null

    open fun memo(): String? = null

    override fun hashCode(): Int {
        return asset.id.toIdentifier().hashCode() +
                destination().hashCode() +
                memo().hashCode() +
                amount.hashCode() +
                useMaxAmount.hashCode()
    }

    companion object {
        fun unpack(input: String): ConfirmParams? {
            return runCatching {
                val json = String(Base64.getDecoder().decode(input.urlDecode()))
                when (JSONObject(json).getString("type")) {
                    TransferParams.Generic::class.qualifiedName -> jsonEncoder.decodeFromString<TransferParams.Generic>(json)
                    TransferParams.Native::class.qualifiedName -> jsonEncoder.decodeFromString<TransferParams.Native>(json)
                    TransferParams.Token::class.qualifiedName -> jsonEncoder.decodeFromString<TransferParams.Token>(json)
                    SwapParams::class.qualifiedName -> jsonEncoder.decodeFromString<SwapParams>(json)
                    TokenApprovalParams::class.qualifiedName -> jsonEncoder.decodeFromString<TokenApprovalParams>(json)
                    Stake.DelegateParams::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.DelegateParams>(json)
                    Stake.UndelegateParams::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.UndelegateParams>(json)
                    Stake.RewardsParams::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.RewardsParams>(json)
                    Stake.RedelegateParams::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.RedelegateParams>(json)
                    Stake.WithdrawParams::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.WithdrawParams>(json)
                    Stake.Freeze::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.Freeze>(json)
                    Stake.Unfreeze::class.qualifiedName -> jsonEncoder.decodeFromString<Stake.Unfreeze>(json)
                    Activate::class.qualifiedName -> jsonEncoder.decodeFromString<Activate>(json)
                    NftParams::class.qualifiedName -> jsonEncoder.decodeFromString<NftParams>(json)
                    PerpetualParams::class.qualifiedName -> jsonEncoder.decodeFromString<PerpetualParams>(json)
                    else -> null
                }
            }.getOrNull()
        }
    }
}

fun GemApprovalData.toModel(): ApprovalData {
    return ApprovalData(
        token = this.token,
        spender = this.spender,
        value = this.value,
        isUnlimited = this.isUnlimited,
    )
}
