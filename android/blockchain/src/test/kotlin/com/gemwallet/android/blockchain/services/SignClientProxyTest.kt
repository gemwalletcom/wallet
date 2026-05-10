package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.clients.SignClient
import com.gemwallet.android.blockchain.clients.cosmos.CosmosChainData
import com.gemwallet.android.model.ChainSignData
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.SwapperProvider
import java.math.BigInteger

class SignClientProxyTest {

    @Test
    fun `transfer swap uses fee adjusted final amount`() = runBlocking {
        val chain = Chain.Thorchain
        val fromAsset = mockAsset(chain = chain, name = "THORChain", symbol = "RUNE")
        val toAsset = mockAsset(chain = Chain.Cosmos, name = "Cosmos", symbol = "ATOM", decimals = 6)
        val fromAmount = BigInteger("2564989685")
        val finalAmount = BigInteger("2562989685")
        val client = RecordingSignClient(chain)
        val params = ConfirmParams.SwapParams(
            from = mockAccount(chain = chain, address = "thor1sender"),
            fromAsset = fromAsset,
            fromAmount = fromAmount,
            toAsset = toAsset,
            toAmount = BigInteger("1000000"),
            swapData = "=:o:cosmos1recipient:0/1/0:g1:50",
            memo = "=:o:cosmos1recipient:0/1/0:g1:50",
            providerId = SwapperProvider.THORCHAIN,
            providerName = "THORChain",
            protocol = "THORChain",
            protocolId = "thorchain",
            toAddress = "thor1vault",
            value = "0",
            slippageBps = 50u,
            etaInSeconds = null,
            dataType = GemSwapQuoteDataType.TRANSFER,
            useMaxAmount = true,
        )
        val signerParams = SignerParams(
            input = params,
            selectedData = SignerParams.Data(
                fee = Fee.Regular(
                    feeAssetId = AssetId(chain),
                    priority = FeePriority.Normal,
                    amount = BigInteger("2000000"),
                    maxGasPrice = BigInteger.ONE,
                    limit = BigInteger("200000"),
                    options = emptyMap(),
                ),
                chainData = CosmosChainData(
                    chainId = "thorchain-mainnet-v1",
                    accountNumber = 1uL,
                    sequence = 3uL,
                ),
            ),
            feeRates = emptyList(),
            finalAmount = finalAmount,
        )

        SignClientProxy(listOf(client)).signTransaction(signerParams, byteArrayOf())

        assertEquals(finalAmount, client.nativeTransferFinalAmount)
        assertEquals(finalAmount, client.nativeTransferParams?.amount)
        assertEquals("thor1vault", client.nativeTransferParams?.destination?.address)
        assertEquals("=:o:cosmos1recipient:0/1/0:g1:50", client.nativeTransferParams?.memo)
    }

    private class RecordingSignClient(private val chain: Chain) : SignClient {
        var nativeTransferParams: ConfirmParams.TransferParams.Native? = null
        var nativeTransferFinalAmount: BigInteger? = null

        override suspend fun signNativeTransfer(
            params: ConfirmParams.TransferParams.Native,
            chainData: ChainSignData,
            finalAmount: BigInteger,
            fee: Fee,
            privateKey: ByteArray,
        ): List<ByteArray> {
            nativeTransferParams = params
            nativeTransferFinalAmount = finalAmount
            return emptyList()
        }

        override fun supported(chain: Chain): Boolean = this.chain == chain
    }
}
