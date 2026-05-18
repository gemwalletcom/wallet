package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.clients.bitcoin.BitcoinGatewayEstimateFee
import com.gemwallet.android.blockchain.gemstone.selectFeeRate
import com.gemwallet.android.blockchain.gemstone.toChainData
import com.gemwallet.android.blockchain.gemstone.toFee
import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.ext.toFeePriority
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.FeePriority
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemGatewayEstimateFee
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionPreloadInput

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
        val selectedRate = validFeeRates.selectFeeRate(feePriority)
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
        val fee = chain.toFee(feeAssetId, selectedPriority, result.fee)
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
            ChainType.Ethereum,
            ChainType.Cardano,
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
            ChainType.Polkadot,
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
