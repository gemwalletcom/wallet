package com.gemwallet.android.blockchain.services

import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.FeePriority
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemGatewayInterface
import uniffi.gemstone.GemTransactionData
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadInput
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger

class SignerPreloaderProxyTest {

    private val gateway = mockk<GemGatewayInterface>()
    private val subject = SignerPreloaderProxy(gateway)

    @Test
    fun preload_loadsOnlySelectedPriorityAndKeepsAllFeeRates() = runBlocking {
        val params = transferParams()
        val metadata = GemTransactionLoadMetadata.Evm(
            nonce = 7u,
            chainId = 1u,
            contractCall = null,
        )
        val feeRates = listOf(
            GemFeeRate(FeePriority.Slow.string, GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "1")),
            GemFeeRate(FeePriority.Normal.string, GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3")),
            GemFeeRate(FeePriority.Fast.string, GemGasPriceType.Eip1559(gasPrice = "4", priorityFee = "5")),
        )
        val loadInput = slot<GemTransactionLoadInput>()

        coEvery { gateway.getTransactionPreload(any(), any()) } returns metadata
        coEvery { gateway.getFeeRates(any(), any()) } returns feeRates
        coEvery { gateway.getTransactionLoad(any(), capture(loadInput), any()) } returns GemTransactionData(
            fee = GemTransactionLoadFee(
                fee = "21000",
                gasPriceType = feeRates[1].gasPriceType,
                gasLimit = "21000",
                options = GemFeeOptions(emptyMap()),
            ),
            metadata = metadata,
        )

        val result = subject.preload(params, FeePriority.Normal)

        assertEquals(feeRates, result.feeRates)
        assertEquals(FeePriority.Normal, result.fee().priority)
        assertEquals(BigInteger("21000"), result.fee().amount)
        assertEquals(feeRates[1].gasPriceType, loadInput.captured.gasPrice)
        coVerify(exactly = 1) { gateway.getTransactionPreload(any(), any()) }
        coVerify(exactly = 1) { gateway.getFeeRates(any(), any()) }
        coVerify(exactly = 1) { gateway.getTransactionLoad(any(), any(), any()) }
    }

    @Test
    fun preload_fallsBackToFirstAvailableValidPriority() = runBlocking {
        val params = transferParams()
        val metadata = GemTransactionLoadMetadata.Evm(
            nonce = 7u,
            chainId = 1u,
            contractCall = null,
        )
        val feeRates = listOf(
            GemFeeRate(priority = "unsupported", gasPriceType = GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "1")),
            GemFeeRate(FeePriority.Fast.string, GemGasPriceType.Eip1559(gasPrice = "4", priorityFee = "5")),
        )
        val loadInput = slot<GemTransactionLoadInput>()

        coEvery { gateway.getTransactionPreload(any(), any()) } returns metadata
        coEvery { gateway.getFeeRates(any(), any()) } returns feeRates
        coEvery { gateway.getTransactionLoad(any(), capture(loadInput), any()) } returns GemTransactionData(
            fee = GemTransactionLoadFee(
                fee = "21000",
                gasPriceType = feeRates[1].gasPriceType,
                gasLimit = "21000",
                options = GemFeeOptions(emptyMap()),
            ),
            metadata = metadata,
        )

        val result = subject.preload(params, FeePriority.Normal)

        assertEquals(listOf(feeRates[1]), result.feeRates)
        assertEquals(FeePriority.Fast, result.fee().priority)
        assertEquals(feeRates[1].gasPriceType, loadInput.captured.gasPrice)
        coVerify(exactly = 1) { gateway.getTransactionLoad(any(), any(), any()) }
    }

    private fun transferParams(): ConfirmParams {
        val asset = mockAssetEthereum()
        val account = mockAccount(
            chain = asset.id.chain,
            address = "0xsender",
            derivationPath = "m/44'/60'/0'/0/0",
        )

        return ConfirmParams.Builder(
            asset = asset,
            from = account,
            amount = BigInteger("1000000000000000"),
        ).transfer(
            destination = DestinationAddress("0xrecipient"),
        )
    }
}
