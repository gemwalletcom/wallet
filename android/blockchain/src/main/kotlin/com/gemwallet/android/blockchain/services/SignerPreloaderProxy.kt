package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.selectFeeRate
import com.gemwallet.android.blockchain.gemstone.toFee
import com.gemwallet.android.blockchain.gemstone.toScanTransactionPayload
import com.gemwallet.android.blockchain.gemstone.validate
import com.gemwallet.android.ext.toFeePriority
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ScanTransaction
import com.wallet.core.primitives.ScanTransactionPayload
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionPreloadInput

class SignerPreloaderProxy(
    private val gateway: GemGatewayInterface,
    private val scanTransaction: suspend (ScanTransactionPayload) -> ScanTransaction? = { null },
) {

    suspend fun preload(params: ConfirmParams, selection: FeeSelection): SignerParams = withContext(Dispatchers.IO) {
        val assetId = params.assetId
        val chain = assetId.chain
        val gemChain = assetId.chain.string
        val destination = requireNotNull(params.destination()?.address)

        val inputType = params.toDto()
        val scanPayload = params.toScanTransactionPayload(destination)
        coroutineScope {
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
            val scanDeferred = async {
                runCatching {
                    scanTransaction(scanPayload)
                }.getOrNull()
            }
            val metadata = metadataDeferred.await()
            val feeRates = feeRatesDeferred.await()
            val validFeeRates = feeRates.filter { it.priority.toFeePriority() != null }
            val selectedRate = validFeeRates.selectFeeRate(selection)
            val selectedPriority = requireNotNull(selectedRate.priority.toFeePriority())

            val transactionLoadDeferred = async {
                gateway.getTransactionLoad(
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
                )
            }
            scanDeferred.await()?.validate(params)
            val result = transactionLoadDeferred.await()
            val fee = result.fee.toFee(selectedPriority)

            SignerParams(
                input = params,
                selectedData = SignerParams.Data(metadata = result.metadata, fee = fee),
                feeRates = validFeeRates,
            )
        }
    }
}
