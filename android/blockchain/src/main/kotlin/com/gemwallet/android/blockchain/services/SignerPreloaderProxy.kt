package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.clients.algorand.toChainData
import com.gemwallet.android.blockchain.clients.aptos.toChainData
import com.gemwallet.android.blockchain.clients.bitcoin.BitcoinGatewayEstimateFee
import com.gemwallet.android.blockchain.clients.bitcoin.toChainData
import com.gemwallet.android.blockchain.clients.cardano.CardanoGatewayEstimateFee
import com.gemwallet.android.blockchain.clients.cardano.toChainData
import com.gemwallet.android.blockchain.clients.cosmos.toChainData
import com.gemwallet.android.blockchain.clients.ethereum.toChainData
import com.gemwallet.android.blockchain.clients.hyper.toChainData
import com.gemwallet.android.blockchain.clients.near.toChainData
import com.gemwallet.android.blockchain.clients.polkadot.PolkadotGatewayEstimateFee
import com.gemwallet.android.blockchain.clients.polkadot.toChainData
import com.gemwallet.android.blockchain.clients.solana.toChainData
import com.gemwallet.android.blockchain.clients.stellar.toChainData
import com.gemwallet.android.blockchain.clients.sui.toChainData
import com.gemwallet.android.blockchain.clients.ton.toChainData
import com.gemwallet.android.blockchain.clients.tron.toChainData
import com.gemwallet.android.blockchain.clients.xrp.toChainData
import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.FeePriority
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemGatewayEstimateFee
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.GemTransactionPreloadInput
import uniffi.gemstone.SwapperException.NotSupportedChain

class SignerPreloaderProxy(
    private val gateway: GemGatewayInterface,
) {

    suspend fun preload(params: ConfirmParams, feePriority: FeePriority): SignerParams = withContext(Dispatchers.IO) {
        val assetId = params.assetId
        val chain = assetId.chain
        val feeAssetId = AssetId(chain)
        val gemChain = assetId.chain.string
        val destination = params.destination()?.address ?: throw java.lang.IllegalArgumentException()

        val inputType = params.toDto()
        val (metadata, feeRates) = coroutineScope {
            val metadataDeferred = async {
                gateway.getTransactionPreload(
                    chain = gemChain,
                    input = GemTransactionPreloadInput(
                        inputType = inputType,
                        senderAddress = params.from.address,
                        destinationAddress = destination
                    )
                )
            }
            val feeRatesDeferred = async {
                gateway.getFeeRates(
                    chain = gemChain,
                    input = inputType
                )
            }
            Pair(metadataDeferred.await(), feeRatesDeferred.await())
        }
        val validFeeRates = feeRates.filter { it.priority.toFeePriority() != null }
        val selectedRate = validFeeRates.select(feePriority)
        val selectedPriority = requireNotNull(selectedRate.priority.toFeePriority())

        val result = gateway.getTransactionLoad(
            chain = gemChain,
            input = GemTransactionLoadInput(
                inputType = inputType,
                senderAddress = params.from.address,
                destinationAddress = destination,
                value = params.amount.toString(),
                gasPrice = selectedRate.gasPriceType,
                memo = params.memo(),
                isMaxValue = params.useMaxAmount,
                metadata = metadata,
            ),
            provider = getEstimateFee(chain)
        )
        val fee = chain.toFeeType().convertFee(feeAssetId, selectedPriority, result.fee)
        val chainData = result.metadata.toChainData()

        SignerParams(
            input = params,
            selectedData = SignerParams.Data(chainData = chainData, fee = fee),
            feeRates = validFeeRates,
        )
    }


    private fun getEstimateFee(chain: Chain): GemGatewayEstimateFee {
        return when (chain.toChainType()) {
            ChainType.Bitcoin -> BitcoinGatewayEstimateFee()
            ChainType.Cardano -> CardanoGatewayEstimateFee()
            ChainType.Polkadot -> PolkadotGatewayEstimateFee()
            ChainType.Ethereum,
            ChainType.Solana,
            ChainType.Cosmos,
            ChainType.Ton,
            ChainType.Tron,
            ChainType.Aptos,
            ChainType.Sui,
            ChainType.Xrp,
            ChainType.Near,
            ChainType.Stellar,
            ChainType.Algorand,
            ChainType.HyperCore -> StubGatewayEstimateFee
        }
    }

    private object StubGatewayEstimateFee : GemGatewayEstimateFee {
        override suspend fun getFee(
            chain: uniffi.gemstone.Chain,
            input: GemTransactionLoadInput
        ): GemTransactionLoadFee? = null

        override suspend fun getFeeData(
            chain: uniffi.gemstone.Chain,
            input: GemTransactionLoadInput
        ): String? = null

    }
}

private fun String.toFeePriority(): FeePriority? = FeePriority.entries.firstOrNull { it.string == this }

private fun List<GemFeeRate>.select(priority: FeePriority): GemFeeRate {
    return firstOrNull { it.priority.toFeePriority() == priority }
        ?: firstOrNull()
        ?: throw IllegalStateException("Fee rates not found")
}

private sealed interface FeeType {

    fun convertFee(feeAssetId: AssetId, priority: FeePriority, gemFee: GemTransactionLoadFee): Fee

    object Plain : FeeType {

        override fun convertFee(
            feeAssetId: AssetId,
            priority: FeePriority,
            gemFee: GemTransactionLoadFee
        ): Fee {
            return Fee.Plain(
                feeAssetId = feeAssetId,
                priority = priority,
                amount = gemFee.fee.toBigInteger(),
                options = gemFee.options.options.mapKeys { it.key.name }.mapValues { it.value.toBigInteger() } // TODO: Review keys
            )
        }

    }
    object Regular : FeeType {

        override fun convertFee(
            feeAssetId: AssetId,
            priority: FeePriority,
            gemFee: GemTransactionLoadFee
        ): Fee {
            return Fee.Regular(
                feeAssetId = feeAssetId,
                priority = priority,
                maxGasPrice = (gemFee.gasPriceType as GemGasPriceType.Regular).gasPrice.toBigInteger(),
                limit = gemFee.gasLimit.toBigInteger(),
                amount = gemFee.fee.toBigInteger(),
                options = gemFee.options.options.mapKeys { it.key.name }.mapValues { it.value.toBigInteger() } // TODO: Review keys
            )
        }

    }
    object Eip1559 : FeeType {

        override fun convertFee(
            feeAssetId: AssetId,
            priority: FeePriority,
            gemFee: GemTransactionLoadFee
        ): Fee {
            return (gemFee.gasPriceType as GemGasPriceType.Eip1559).let { price ->
                Fee.Eip1559(
                    feeAssetId = feeAssetId,
                    priority = priority,
                    maxGasPrice = price.gasPrice.toBigInteger(),
                    minerFee = price.priorityFee.toBigInteger(),
                    limit = gemFee.gasLimit.toBigInteger(),
                    amount = gemFee.fee.toBigInteger(),
                    options = gemFee.options.options.mapKeys { it.key.name }.mapValues { it.value.toBigInteger() } // TODO: Change keys
                )
            }
        }
    }

    object Solana : FeeType {

        override fun convertFee(
            feeAssetId: AssetId,
            priority: FeePriority,
            gemFee: GemTransactionLoadFee
        ): Fee {
            return (gemFee.gasPriceType as GemGasPriceType.Solana).let { price ->
                Fee.Solana(
                    feeAssetId = feeAssetId,
                    priority = priority,
                    maxGasPrice = price.gasPrice.toBigInteger(),
                    minerFee = price.priorityFee.toBigInteger(),
                    unitFee = price.unitPrice.toBigInteger(),
                    limit = gemFee.gasLimit.toBigInteger(),
                    amount = gemFee.fee.toBigInteger(),
                    options = gemFee.options.options.mapKeys { it.key.name }.mapValues { it.value.toBigInteger() } // TODO: Change keys
                )
            }
        }
    }
}

private fun Chain.toFeeType() = when (this.toChainType()) {
    ChainType.Solana -> FeeType.Solana
    ChainType.Bitcoin,
    ChainType.Cosmos,
    ChainType.Tron,
    ChainType.Aptos -> FeeType.Regular
    ChainType.Ethereum -> FeeType.Eip1559
    ChainType.HyperCore,
    ChainType.Ton,
    ChainType.Sui,
    ChainType.Xrp,
    ChainType.Near,
    ChainType.Stellar,
    ChainType.Algorand,
    ChainType.Polkadot,
    ChainType.Cardano -> FeeType.Plain
}

private fun GemTransactionLoadMetadata.toChainData() = when (this) {
    is GemTransactionLoadMetadata.Algorand -> toChainData()
    is GemTransactionLoadMetadata.Aptos -> toChainData()
    is GemTransactionLoadMetadata.Bitcoin -> toChainData()
    is GemTransactionLoadMetadata.Zcash -> toChainData()
    is GemTransactionLoadMetadata.Cardano -> toChainData()
    is GemTransactionLoadMetadata.Cosmos -> toChainData()
    is GemTransactionLoadMetadata.Evm -> toChainData()
    is GemTransactionLoadMetadata.Near -> toChainData()
    is GemTransactionLoadMetadata.Polkadot -> toChainData()
    is GemTransactionLoadMetadata.Solana -> toChainData()
    is GemTransactionLoadMetadata.Stellar -> toChainData()
    is GemTransactionLoadMetadata.Sui -> toChainData()
    is GemTransactionLoadMetadata.Ton -> toChainData()
    is GemTransactionLoadMetadata.Tron -> toChainData()
    is GemTransactionLoadMetadata.Xrp -> toChainData()
    is GemTransactionLoadMetadata.Hyperliquid -> toChainData()
    GemTransactionLoadMetadata.None -> throw NotSupportedChain()
}
